use super::*;
use fs2::FileExt;

/*pub fn load_env() -> Result<(String, String, String, String, String), String> {
    if let Err(result) = dotenv::dotenv() {
        return Err(error_message!(
            "failed to load .env file!\ndetails : {:?}",
            result
        ));
    }

    let log_dir = match std::env::var(common_constants::ENV_NAME_ZENIES_LOG_DIR) {
        Ok(result) => result,
        Err(result) => {
            return Err(error_message!(
                "not found {}\ndetails : {:?}",
                common_constants::ENV_NAME_ZENIES_LOG_DIR,
                result
            ));
        }
    };

    let db_path = match std::env::var(common_constants::ENV_NAME_ZENIES_DATABASE_PATH) {
        Ok(result) => result,
        Err(result) => {
            return Err(error_message!(
                "not found {}\ndetails : {:?}",
                common_constants::ENV_NAME_ZENIES_DATABASE_PATH,
                result
            ));
        }
    };

    let etc_dir = match std::env::var(common_constants::ENV_NAME_ZENIES_ETC_DIR) {
        Ok(result) => result,
        Err(result) => {
            return Err(error_message!(
                "not found {}\ndetails : {:?}",
                common_constants::ENV_NAME_ZENIES_ETC_DIR,
                result
            ));
        }
    };

    let bin_dir = match std::env::var(common_constants::ENV_NAME_ZENIES_BIN_DIR) {
        Ok(result) => result,
        Err(result) => {
            return Err(error_message!(
                "not found {}\ndetails : {:?}",
                common_constants::ENV_NAME_ZENIES_BIN_DIR,
                result
            ));
        }
    };

    let keybox_path = match std::env::var(common_constants::ENV_NAME_ZENIES_KEY_BOX_PATH) {
        Ok(result) => result,
        Err(result) => {
            return Err(error_message!(
                "not found {}\ndetails : {:?}",
                common_constants::ENV_NAME_ZENIES_BIN_DIR,
                result
            ));
        }
    };

    Ok((log_dir, db_path, etc_dir, bin_dir, keybox_path))
}*/

pub fn lock_file(etc_dir: &str) -> Result<(), String> {
    let lock_file = match std::fs::File::create(
        String::from(etc_dir) + env!("CARGO_PKG_NAME") + common_constants::LOCK_EXTENSION,
    ) {
        Ok(result) => result,
        Err(result) => {
            return Err(error_message!(
                "failed to open a lock file!\ndetails : {:?}",
                result
            ));
        }
    };
    if let Err(result) = lock_file.try_lock_exclusive() {
        return Err(error_message!(
            "failed to lock a lock file!\ndetails :{:?}",
            result
        ));
    }
    Ok(())
}
