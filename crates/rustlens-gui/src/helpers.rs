use rustlens_core::model::connection::ConnectionProfile;

use crate::{
    profiles, AddExistingForm, CreateEngine, CreateLocalForm, EditProfileForm, DEFAULT_SCHEMA,
};

pub(crate) fn build_manual_profile(form: &AddExistingForm) -> Result<ConnectionProfile, String> {
    let page_size = form
        .page_size
        .parse::<i64>()
        .map_err(|_| "Page size must be a number.".to_string())?;
    if page_size <= 0 {
        return Err("Page size must be greater than 0.".to_string());
    }

    let database_url = if form.advanced_url {
        if form.database_url.trim().is_empty() {
            return Err("Database URL is required.".to_string());
        }
        form.database_url.trim().to_string()
    } else {
        let port = form
            .port
            .parse::<u16>()
            .map_err(|_| "Port must be a number between 1 and 65535.".to_string())?;
        if form.host.trim().is_empty()
            || form.database.trim().is_empty()
            || form.user.trim().is_empty()
        {
            return Err("Host, database, and user are required.".to_string());
        }

        format!(
            "postgres://{}:{}@{}:{}/{}",
            urlencoding::encode(form.user.trim()),
            urlencoding::encode(&form.password),
            form.host.trim(),
            port,
            urlencoding::encode(form.database.trim())
        )
    };

    Ok(profiles::manual_postgres_profile(
        form.name.trim().to_string(),
        database_url,
        empty_to_default(&form.schema, DEFAULT_SCHEMA),
        page_size,
    ))
}

pub(crate) fn edit_form_from_profile(profile: &profiles::Profile) -> EditProfileForm {
    EditProfileForm {
        id: profile.id,
        name: profile.name.clone(),
        driver: profile.driver,
        database_url: profile.database_url.clone(),
        schema: profile.schema.clone(),
        page_size: profile.page_size.to_string(),
        source: profile.source,
        dbnest_instance_id: profile.dbnest_instance_id.clone(),
    }
}

pub(crate) fn build_profile_from_edit(form: &EditProfileForm) -> Result<ConnectionProfile, String> {
    if form.name.trim().is_empty() {
        return Err("Profile name is required.".to_string());
    }
    if form.database_url.trim().is_empty() {
        return Err("Database URL is required.".to_string());
    }
    let page_size = form
        .page_size
        .parse::<i64>()
        .map_err(|_| "Page size must be a number.".to_string())?;
    if page_size <= 0 {
        return Err("Page size must be greater than 0.".to_string());
    }

    Ok(ConnectionProfile {
        id: form.id,
        name: form.name.trim().to_string(),
        driver: form.driver,
        database_url: form.database_url.trim().to_string(),
        schema: Some(empty_to_default(&form.schema, DEFAULT_SCHEMA)),
        page_size: Some(page_size),
        source: form.source,
        dbnest_instance_id: form.dbnest_instance_id.clone(),
    })
}

pub(crate) fn create_profile_with_dbnest(
    form: CreateLocalForm,
) -> anyhow::Result<ConnectionProfile> {
    match form.engine {
        CreateEngine::PostgresDocker => {
            let page_size = form.page_size.parse::<i64>()?;
            rustlens_core::provision::dbnest::create_postgres_docker(
                rustlens_core::provision::dbnest::CreatePostgresDocker {
                    profile_name: empty_to_default(&form.profile_name, "local"),
                    user: empty_to_default(&form.user, "app"),
                    password: empty_to_default(&form.password, "app"),
                    database: empty_to_default(&form.database, "appdb"),
                    image: if form.image.trim().is_empty() {
                        None
                    } else {
                        Some(form.image.trim().to_string())
                    },
                    schema: empty_to_default(&form.schema, DEFAULT_SCHEMA),
                    page_size,
                },
            )
        }
        CreateEngine::Sqlite => rustlens_core::provision::dbnest::create_sqlite(
            rustlens_core::provision::dbnest::CreateSqlite {
                profile_name: empty_to_default(&form.profile_name, "sqlite"),
                path: if form.sqlite_path.trim().is_empty() {
                    None
                } else {
                    Some(std::path::PathBuf::from(form.sqlite_path.trim()))
                },
            },
        ),
        CreateEngine::Mysql => {
            anyhow::bail!("MySQL/MariaDB provisioning is planned but not implemented yet")
        }
    }
}

fn empty_to_default(value: &str, default: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        default.to_string()
    } else {
        trimmed.to_string()
    }
}
