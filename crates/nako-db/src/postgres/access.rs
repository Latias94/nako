use nako_core::{AuthenticatedPrincipal, MediaItemId};
use sqlx::{Postgres, QueryBuilder};

pub(crate) fn push_media_item_access_filter(
    query: &mut QueryBuilder<'_, Postgres>,
    principal: &AuthenticatedPrincipal,
    item_id_sql: &str,
) {
    if principal.is_administrator() {
        return;
    }

    query.push("\n              AND ");
    push_media_item_access_exists(query, principal, item_id_sql);
}

pub(crate) fn push_media_item_access_exists(
    query: &mut QueryBuilder<'_, Postgres>,
    principal: &AuthenticatedPrincipal,
    item_id_sql: &str,
) {
    query.push(
        r#"
EXISTS (
    SELECT 1
    FROM media_sources AS access_sources
    WHERE access_sources.item_id = "#,
    );
    query.push(item_id_sql);
    query.push(
        r#"
      AND (
          EXISTS (
              SELECT 1
              FROM user_library_access_policies AS user_policies
              WHERE user_policies.user_id = "#,
    );
    query.push_bind(principal.user_id.as_uuid());
    query.push(
        r#"
                AND user_policies.library_id = access_sources.library_id
                AND user_policies.access IN ('browse', 'play', 'manage')
          )
"#,
    );

    if !principal.roles.is_empty() {
        query.push(
            r#"
          OR EXISTS (
              SELECT 1
              FROM role_library_access_policies AS role_policies
              WHERE role_policies.library_id = access_sources.library_id
                AND role_policies.access IN ('browse', 'play', 'manage')
                AND role_policies.role IN ("#,
        );
        let mut separated = query.separated(", ");
        for role in &principal.roles {
            separated.push_bind(role.as_str());
        }
        drop(separated);
        query.push(
            r#"
                )
          )
"#,
        );
    }

    query.push(
        r#"
      )
)
"#,
    );
}

pub(crate) fn push_media_item_id_filter(
    query: &mut QueryBuilder<'_, Postgres>,
    item_ids: &[MediaItemId],
    item_id_sql: &str,
) {
    query.push("\n              AND ");
    query.push(item_id_sql);
    query.push(" IN (");
    let mut separated = query.separated(", ");
    for item_id in item_ids {
        separated.push_bind(item_id.as_uuid());
    }
    drop(separated);
    query.push(")");
}
