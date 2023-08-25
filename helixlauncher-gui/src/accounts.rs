use helixlauncher_core::auth::account::AccountConfig;
use helixlauncher_core::auth::{MinecraftAuthenticator, DEFAULT_ACCOUNT_JSON};
use helixlauncher_core::config::Config;
use helixlauncher_core::launch::{
    asset::merge_components,
    instance::{Instance, InstanceLaunchConfig, Modloader},
    prepared::{prepare_launch, LaunchOptions},
};
use qmetaobject::USER_ROLE;
use qmetaobject::{prelude::*, QSingletonInit};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, RwLock, Mutex};
use std::time::Duration;
use tokio::runtime::{self, Runtime};

#[derive(Default, QObject)]
pub struct SignInModel {
    base: qt_base_class!(trait QObject),
    get_message: qt_method!(fn(&self) -> QString),
    //auth_callback: fn(&mut self),
}

impl SignInModel {

    fn get_message(&mut self) -> QString {
        let minecraft_authenticator: MinecraftAuthenticator =
            MinecraftAuthenticator::new("1d644380-5a23-4a84-89c3-5d29615fbac2");
        //let mut message = Arc::new(String::new());
        let m = Arc::new(Mutex::new(String::new()));
        let t = std::thread::spawn({
            let m = Arc::clone(&m);
            move || {
                let rt = Runtime::new().unwrap();
                rt.block_on(async {
                    minecraft_authenticator.initial_auth(|code, uri, message| {
                        println!("{code}, {uri}, {message}");
                        let mut inner_m = m.lock().unwrap();
                        *inner_m = message;
                    }).await.unwrap();
                
                });
                
            }
        });
        //let ma = m.lock().unwrap();
        //let message = ma.clone().to_string();
        let rt = Runtime::new().unwrap();
        loop {
            rt.block_on(async {
                tokio::time::sleep(Duration::from_secs(1)).await;
            });
            println!("hello world");
            if *m.lock().unwrap() != "".to_string() {
                println!("a");
                println!("test: {}", *m.lock().unwrap());
                break
            } else {
                println!("{}", *m.lock().unwrap())
            }
        }
        //t.join().unwrap();

        //println!("{}", ); 

        // <message as a string>.into()
        let a = m.lock().unwrap();
        let b = a.clone();
        b.into()
    }
}

#[derive(Default, QObject)]
pub struct AccountsModel {
    base: qt_base_class!(trait QAbstractListModel),
    remove: qt_method!(fn(&self, uuid: QString)),
    set_default: qt_method!(fn(&self, uuid: QString)),
}

impl AccountsModel {
    fn remove(&self, uuid: QString) {
        let true_uuid: String = uuid.into();
        let config = Config::new("dev.helixlauncher.HelixLauncher", "HelixLauncher").unwrap();
        let base_path = config.get_base_path();
        let mut account_config = AccountConfig::new(base_path.join(DEFAULT_ACCOUNT_JSON)).unwrap();
        let mut accounts = account_config.clone().accounts;
        let account = accounts.iter().position(|x| x.uuid == true_uuid);
        if let Some(account_some) = account {
            accounts.remove(account_some);
            account_config.accounts = accounts.clone();
            if accounts.is_empty() {
                account_config.default = None
            }
            account_config.save().unwrap()
        } else {
            ()
        }
    }

    fn set_default(&self, uuid: QString) {
        let true_uuid: String = uuid.into();
        let config = Config::new("dev.helixlauncher.HelixLauncher", "HelixLauncher").unwrap();
        let base_path = config.get_base_path();
        let mut account_config = AccountConfig::new(base_path.join(DEFAULT_ACCOUNT_JSON)).unwrap();
        account_config.default = Some(true_uuid);
        account_config.save().unwrap()
    }
}

impl QAbstractListModel for AccountsModel {
    fn row_count(&self) -> i32 {
        let config = Config::new("dev.helixlauncher.HelixLauncher", "HelixLauncher").unwrap();
        let base_path = config.get_base_path();
        let account_config = AccountConfig::new(base_path.join(DEFAULT_ACCOUNT_JSON)).unwrap();
        let accounts = account_config.clone().accounts;
        //println!("{:#?}", accounts);
        //println!("{:#?}", accounts.len());
        //println!("{:#?}", account_config);
        accounts.len() as _
    }

    fn data(&self, index: QModelIndex, role: i32) -> QVariant {
        let config = Config::new("dev.helixlauncher.HelixLauncher", "HelixLauncher").unwrap();
        let base_path = config.get_base_path();
        let account_config = AccountConfig::new(base_path.join(DEFAULT_ACCOUNT_JSON)).unwrap();
        let mut accounts = account_config.accounts;
        accounts.sort_by(|x, y| x.username.cmp(&y.username));

        if let Some(elem) = accounts.get(index.row() as usize) {
            if role == USER_ROLE {
                println!(
                    "{:#?}",
                    Into::<QVariant>::into(QString::from(elem.username.as_str()))
                );
                QString::from(elem.username.as_str()).into()
            } else if role == USER_ROLE + 1 {
                QString::from(elem.uuid.as_str()).into()
            } else {
                QVariant::default()
            }
        } else {
            QVariant::default()
        }
    }

    fn role_names(&self) -> HashMap<i32, QByteArray> {
        let mut map = HashMap::new();
        map.insert(USER_ROLE, "username".into());
        map.insert(USER_ROLE + 1, "uuid".into());
        map
    }
}

impl QSingletonInit for AccountsModel {
    fn init(&mut self) {}
}

impl QSingletonInit for SignInModel {
    fn init(&mut self) {}
}
