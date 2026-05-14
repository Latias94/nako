use taru_core::LibraryId;

fn main() {
    let server_boot_id = LibraryId::new();

    tracing::info!(%server_boot_id, "starting Taru server foundation");
    println!("taru server foundation {server_boot_id}");
}
