use nako_addon_protocol::{
    AddonInstallDescriptor, AddonManifest, addon_install_guide as protocol_addon_install_guide,
    validate_install_descriptor, validate_manifest,
};
use nako_api::extension::{
    AdminAddonInstallGuideLifecycleBoundary, AdminAddonInstallGuidePreviewRequest,
    AdminAddonInstallGuidePreviewResponse, AdminAddonInstallGuideResponse,
    AdminAddonInstallGuideSecretReference, AdminAddonInstallGuideSnippet,
    AdminAddonInstallGuideStep, AdminAddonSourceCatalogEntriesResponse,
    AdminAddonSourceCatalogEntry, AdminAddonSourceCatalogResolveResponse,
    AdminAddonSourceCatalogSource, AdminAddonSourceCatalogSourceKind,
    AdminAddonSourceCatalogSourcesResponse,
};
use nako_core::{AddonId, AddonRegistrationRecord, NakoError, Result};
use nako_official_addon_catalog::{
    chromecast_renderer, dlna_renderer, external_acquisition_runner, metadata_scraper,
    notification_bridge, resource_search, subtitle_provider,
};

use super::{AddonAppService, addon_surface_url};
impl AddonAppService {
    pub fn preview_addon_install_guide(
        &self,
        request: AdminAddonInstallGuidePreviewRequest,
    ) -> Result<AdminAddonInstallGuidePreviewResponse> {
        validate_install_descriptor(&request.descriptor).map_err(|_err| {
            NakoError::InvalidInput {
                message: "invalid addon install descriptor".to_owned(),
            }
        })?;

        Ok(AdminAddonInstallGuidePreviewResponse {
            guide: protocol_addon_install_guide(&request.descriptor),
        })
    }

    pub fn list_addon_source_catalog_sources(
        &self,
    ) -> Result<AdminAddonSourceCatalogSourcesResponse> {
        let entries = builtin_addon_catalog_entries()?;
        Ok(AdminAddonSourceCatalogSourcesResponse {
            sources: vec![AdminAddonSourceCatalogSource {
                id: "nako-official".to_owned(),
                name: "Nako Official Addons".to_owned(),
                description: Some(
                    "Built-in source for official Addon Sidecars published for the current alpha"
                        .to_owned(),
                ),
                kind: AdminAddonSourceCatalogSourceKind::BuiltinOfficial,
                entry_count: entries.len(),
                provides_package_signing: false,
                provides_process_supervision: false,
                provides_provider_breadth: false,
            }],
        })
    }

    pub fn list_addon_source_catalog_entries(
        &self,
    ) -> Result<AdminAddonSourceCatalogEntriesResponse> {
        let entries = builtin_addon_catalog_entries()?;
        Ok(AdminAddonSourceCatalogEntriesResponse {
            source_id: "nako-official".to_owned(),
            entries,
        })
    }

    pub fn resolve_addon_source_catalog_entry(
        &self,
        entry_id: &str,
    ) -> Result<AdminAddonSourceCatalogResolveResponse> {
        let descriptor = builtin_addon_catalog_descriptor(entry_id)?;
        let entry = addon_catalog_entry_from_descriptor("nako-official", entry_id, &descriptor);
        validate_install_descriptor(&descriptor).map_err(|_err| NakoError::InvalidInput {
            message: "invalid addon catalog install descriptor".to_owned(),
        })?;
        let install_guide = protocol_addon_install_guide(&descriptor);

        Ok(AdminAddonSourceCatalogResolveResponse {
            source_id: "nako-official".to_owned(),
            entry,
            descriptor,
            install_guide,
        })
    }

    pub async fn get_addon_install_guide(
        &self,
        addon_id: AddonId,
    ) -> Result<AdminAddonInstallGuideResponse> {
        let addon = self.get_addon_registration_or_not_found(addon_id).await?;
        let manifest = self.stored_manifest(&addon)?;
        validate_manifest(&manifest).map_err(|err| NakoError::InvalidInput {
            message: err.to_string(),
        })?;

        Ok(addon_install_guide(&addon, &manifest))
    }
}

fn addon_install_guide(
    addon: &AddonRegistrationRecord,
    manifest: &AddonManifest,
) -> AdminAddonInstallGuideResponse {
    let service_name = addon_service_name(&manifest.id);
    let health_url = addon_surface_url(&manifest.base_url, "/health");
    let secret_references = manifest
        .secret_reference_fields
        .iter()
        .map(|field| AdminAddonInstallGuideSecretReference {
            id: field.id.clone(),
            label: field.label.clone(),
            description: field.description.clone(),
            required: field.required,
            env_var: secret_reference_env_var(&field.id),
            placeholder: format!("secret-reference:{}", field.id),
        })
        .collect::<Vec<_>>();

    AdminAddonInstallGuideResponse {
        addon_id: addon.id,
        manifest_id: addon.manifest_id.clone(),
        addon_name: addon.name.clone(),
        addon_version: addon.version.clone(),
        protocol_version: addon.protocol_version.clone(),
        base_url: addon.base_url.clone(),
        status: addon.status,
        docker_compose: docker_compose_install_snippet(manifest, &service_name, &secret_references),
        systemd: systemd_install_snippet(manifest, &service_name, &secret_references),
        secret_references,
        health_check_steps: vec![
            AdminAddonInstallGuideStep {
                title: "Check the Addon Sidecar health contract directly".to_owned(),
                command: format!(
                    "curl -fsS -X POST {} -H {} -d {}",
                    shell_quote(&health_url),
                    shell_quote("Content-Type: application/json"),
                    shell_quote(&serde_json::json!({
                        "protocol_version": manifest.protocol_version,
                        "manifest_id": manifest.id,
                        "request_id": "manual-health-check",
                        "expected_addon_version": manifest.version,
                        "expected_resource_count": manifest.resources.len()
                    })
                    .to_string())
                ),
                expected_result: "The sidecar returns matching protocol, manifest, addon version, and resource-count facts.".to_owned(),
            },
            AdminAddonInstallGuideStep {
                title: "Check the Addon through Nako Admin API".to_owned(),
                command: format!(
                    "curl -fsS -X POST \"$NAKO_BASE_URL/admin/v1/addons/{}/health-check\" -H {}",
                    addon.id,
                    shell_quote("Authorization: <admin-auth-header>")
                ),
                expected_result: "Nako returns a redaction-safe Addon Health Check status without sending Admin credentials or resolved secrets to the sidecar.".to_owned(),
            },
        ],
        registration_verification_steps: vec![
            AdminAddonInstallGuideStep {
                title: "Verify the registered Addon manifest snapshot".to_owned(),
                command: format!(
                    "curl -fsS \"$NAKO_BASE_URL/admin/v1/addons/{}\" -H {}",
                    addon.id,
                    shell_quote("Authorization: <admin-auth-header>")
                ),
                expected_result: format!(
                    "The response summary contains manifest_id `{}` and status `{}`.",
                    addon.manifest_id,
                    addon.status.as_str()
                ),
            },
            AdminAddonInstallGuideStep {
                title: "Verify declared Addon surfaces".to_owned(),
                command: format!(
                    "curl -fsS \"$NAKO_BASE_URL/admin/v1/addons/{}/surfaces\" -H {}",
                    addon.id,
                    shell_quote("Authorization: <admin-auth-header>")
                ),
                expected_result: "The response lists Entry Points, Hosted Pages, Secret Reference fields, Tasks, and Event Subscriptions as declarations only.".to_owned(),
            },
        ],
        lifecycle_boundary: AdminAddonInstallGuideLifecycleBoundary {
            nako_manages_containers: false,
            nako_manages_processes: false,
            nako_manages_packages: false,
            message: "Nako generates this guide only. The operator owns Addon Sidecar installation, start/stop, upgrades, logs, and removal outside Nako.".to_owned(),
        },
    }
}

fn builtin_addon_catalog_entries() -> Result<Vec<AdminAddonSourceCatalogEntry>> {
    let descriptors = vec![
        (
            metadata_scraper::ADDON_ID,
            official_metadata_scraper_descriptor(),
        ),
        (
            notification_bridge::ADDON_ID,
            official_notification_bridge_descriptor(),
        ),
        (
            chromecast_renderer::ADDON_ID,
            official_chromecast_renderer_descriptor(),
        ),
        (
            resource_search::ADDON_ID,
            official_resource_search_descriptor(),
        ),
        (
            external_acquisition_runner::ADDON_ID,
            official_external_acquisition_runner_descriptor(),
        ),
        (
            subtitle_provider::ADDON_ID,
            official_subtitle_provider_descriptor(),
        ),
        (dlna_renderer::ADDON_ID, official_dlna_renderer_descriptor()),
    ];
    for (_, descriptor) in &descriptors {
        validate_install_descriptor(descriptor).map_err(|_err| NakoError::InvalidInput {
            message: "invalid built-in addon catalog descriptor".to_owned(),
        })?;
    }

    Ok(descriptors
        .into_iter()
        .map(|(entry_id, descriptor)| {
            addon_catalog_entry_from_descriptor("nako-official", entry_id, &descriptor)
        })
        .collect())
}

fn builtin_addon_catalog_descriptor(entry_id: &str) -> Result<AddonInstallDescriptor> {
    match entry_id {
        metadata_scraper::ADDON_ID => Ok(official_metadata_scraper_descriptor()),
        notification_bridge::ADDON_ID => Ok(official_notification_bridge_descriptor()),
        chromecast_renderer::ADDON_ID => Ok(official_chromecast_renderer_descriptor()),
        resource_search::ADDON_ID => Ok(official_resource_search_descriptor()),
        external_acquisition_runner::ADDON_ID => {
            Ok(official_external_acquisition_runner_descriptor())
        }
        subtitle_provider::ADDON_ID => Ok(official_subtitle_provider_descriptor()),
        dlna_renderer::ADDON_ID => Ok(official_dlna_renderer_descriptor()),
        _ => Err(NakoError::NotFound {
            entity: "addon_catalog_entry",
            id: entry_id.to_owned(),
        }),
    }
}

fn addon_catalog_entry_from_descriptor(
    source_id: &str,
    entry_id: &str,
    descriptor: &AddonInstallDescriptor,
) -> AdminAddonSourceCatalogEntry {
    AdminAddonSourceCatalogEntry {
        source_id: source_id.to_owned(),
        entry_id: entry_id.to_owned(),
        manifest_id: descriptor.manifest.id.clone(),
        addon_name: descriptor.manifest.name.clone(),
        addon_version: descriptor.manifest.version.clone(),
        protocol_version: descriptor.manifest.protocol_version.clone(),
        description: descriptor.manifest.description.clone(),
        runtime_kind: descriptor.runtime.kind,
        resources: descriptor
            .manifest
            .resources
            .iter()
            .map(|resource| resource.kind)
            .collect(),
        scopes: descriptor.manifest.scopes.clone(),
        tasks: descriptor
            .manifest
            .tasks
            .iter()
            .map(|task| task.id.clone())
            .collect(),
        package_signing_verified: false,
        lifecycle_boundary: AdminAddonInstallGuideLifecycleBoundary {
            nako_manages_containers: false,
            nako_manages_processes: false,
            nako_manages_packages: false,
            message: "The catalog resolves install metadata only. Operators still own package installation, sidecar process lifecycle, update execution, logs, and rollback outside Nako.".to_owned(),
        },
    }
}

pub(super) fn official_metadata_scraper_descriptor() -> AddonInstallDescriptor {
    metadata_scraper::container_install_descriptor()
}

pub(super) fn official_notification_bridge_descriptor() -> AddonInstallDescriptor {
    notification_bridge::container_install_descriptor()
}

pub(super) fn official_chromecast_renderer_descriptor() -> AddonInstallDescriptor {
    chromecast_renderer::container_install_descriptor()
}

pub(super) fn official_resource_search_descriptor() -> AddonInstallDescriptor {
    resource_search::container_install_descriptor()
}

pub(super) fn official_external_acquisition_runner_descriptor() -> AddonInstallDescriptor {
    external_acquisition_runner::container_install_descriptor()
}

pub(super) fn official_subtitle_provider_descriptor() -> AddonInstallDescriptor {
    subtitle_provider::container_install_descriptor()
}

pub(super) fn official_dlna_renderer_descriptor() -> AddonInstallDescriptor {
    dlna_renderer::container_install_descriptor()
}

fn docker_compose_install_snippet(
    manifest: &AddonManifest,
    service_name: &str,
    secret_references: &[AdminAddonInstallGuideSecretReference],
) -> AdminAddonInstallGuideSnippet {
    let mut environment = vec![
        format!(
            "      NAKO_ADDON_BASE_URL: {}",
            yaml_quote(&manifest.base_url)
        ),
        format!(
            "      NAKO_ADDON_PROTOCOL_VERSION: {}",
            yaml_quote(&manifest.protocol_version)
        ),
        format!("      NAKO_ADDON_MANIFEST_ID: {}", yaml_quote(&manifest.id)),
    ];
    if secret_references.is_empty() {
        environment
            .push("      # No Secret Reference fields are declared by this manifest.".to_owned());
    } else {
        environment.extend(secret_references.iter().map(|secret| {
            format!(
                "      {}: {}",
                secret.env_var,
                yaml_quote(&secret.placeholder)
            )
        }));
    }

    let content = format!(
        r#"services:
  {service_name}:
    image: {image}
    restart: unless-stopped
    environment:
{environment}
    healthcheck:
      test: ["CMD-SHELL", {healthcheck}]
      interval: 30s
      timeout: 5s
      retries: 5
      start_period: 20s
"#,
        image = yaml_quote(&format!(
            "<replace-with-{}-image>:{}",
            service_name, manifest.version
        )),
        environment = environment.join("\n"),
        healthcheck = yaml_quote(&format!(
            "curl -fsS {} >/dev/null",
            addon_surface_url(&manifest.base_url, "/health")
        )),
    );

    AdminAddonInstallGuideSnippet {
        title: "Docker Compose sidecar snippet".to_owned(),
        filename: format!("compose.{service_name}.yml"),
        content,
        notes: vec![
            "Run this Addon Sidecar as a separate service on a network Nako can reach.".to_owned(),
            "Replace the image placeholder with the Addon author's published image.".to_owned(),
            "Nako does not mount the Docker socket or manage this container lifecycle.".to_owned(),
        ],
    }
}

fn systemd_install_snippet(
    manifest: &AddonManifest,
    service_name: &str,
    secret_references: &[AdminAddonInstallGuideSecretReference],
) -> AdminAddonInstallGuideSnippet {
    let mut environment = vec![
        systemd_environment("NAKO_ADDON_BASE_URL", &manifest.base_url),
        systemd_environment("NAKO_ADDON_PROTOCOL_VERSION", &manifest.protocol_version),
        systemd_environment("NAKO_ADDON_MANIFEST_ID", &manifest.id),
    ];
    if secret_references.is_empty() {
        environment.push("# No Secret Reference fields are declared by this manifest.".to_owned());
    } else {
        environment.extend(
            secret_references
                .iter()
                .map(|secret| systemd_environment(&secret.env_var, &secret.placeholder)),
        );
    }

    let content = format!(
        r#"[Unit]
Description={name} Addon Sidecar
After=network-online.target

[Service]
Type=simple
{environment}
ExecStart=<addon-sidecar-command> --listen 0.0.0.0:{port}
Restart=on-failure
RestartSec=5s

[Install]
WantedBy=multi-user.target
"#,
        name = manifest.name,
        environment = environment.join("\n"),
        port = addon_base_url_port(&manifest.base_url),
    );

    AdminAddonInstallGuideSnippet {
        title: "systemd sidecar unit snippet".to_owned(),
        filename: format!("{service_name}.service"),
        content,
        notes: vec![
            "Replace <addon-sidecar-command> with the Addon author's binary and arguments.".to_owned(),
            "Keep Secret Reference placeholders out of this unit until your host secret policy resolves them safely.".to_owned(),
            "Nako does not call systemd or supervise this process.".to_owned(),
        ],
    }
}

fn addon_service_name(manifest_id: &str) -> String {
    let mut output = String::new();
    let mut last_was_dash = false;
    for character in manifest_id.chars() {
        if character.is_ascii_alphanumeric() {
            output.push(character.to_ascii_lowercase());
            last_was_dash = false;
        } else if !last_was_dash {
            output.push('-');
            last_was_dash = true;
        }
    }
    let output = output.trim_matches('-').to_owned();
    if output.is_empty() {
        "nako-addon-sidecar".to_owned()
    } else {
        output
    }
}

fn secret_reference_env_var(id: &str) -> String {
    let mut output = String::from("ADDON_SECRET_");
    let mut last_was_underscore = false;
    for character in id.chars() {
        if character.is_ascii_alphanumeric() {
            output.push(character.to_ascii_uppercase());
            last_was_underscore = false;
        } else if !last_was_underscore {
            output.push('_');
            last_was_underscore = true;
        }
    }

    while output.ends_with('_') {
        output.pop();
    }
    if output == "ADDON_SECRET" {
        "ADDON_SECRET_VALUE".to_owned()
    } else {
        output
    }
}

fn addon_base_url_port(base_url: &str) -> u16 {
    reqwest::Url::parse(base_url)
        .ok()
        .and_then(|url| url.port_or_known_default())
        .unwrap_or(8080)
}

fn yaml_quote(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn systemd_environment(key: &str, value: &str) -> String {
    format!(
        "Environment=\"{key}={}\"",
        value.replace('\\', "\\\\").replace('"', "\\\"")
    )
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}
