use diesel::pg::Pg;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

pub fn run_migrations(connection: &mut impl MigrationHarness<Pg>) {
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("Could not run migrations");
}
