use helixlauncher_core::config::Config;
use helixlauncher_core::launch::{
    asset::merge_components,
    game::{prepare_launch, LaunchOptions},
    instance::{Instance, InstanceLaunch, Modloader}
};
use qmetaobject::USER_ROLE;
use qmetaobject::{prelude::*, QSingletonInit};
use std::collections::HashMap;
use tokio::runtime::Runtime;

#[derive(Default, QObject)]
pub struct InstancesModel {
    base: qt_base_class!(trait QAbstractListModel),

    launch: qt_method!(fn(&self, item: usize)),
    create_instance: qt_method!(
        fn(
            &mut self,
            name: String,
            version: String,
            modloader_string: String,
            modloader_version: String,
        )
    ),
}

impl InstancesModel {
    fn launch(&self, item: usize) {
        std::thread::spawn(move || {
            let config = Config::new("dev.helixlauncher.HelixLauncher", "HelixLauncher").unwrap();
            let mut instances = Instance::list_instances(config.get_instances_path()).unwrap();
            instances.sort_by(|x, y| x.path.cmp(&y.path));

            let instance = &instances[item];

            let rt = Runtime::new().unwrap();
            rt.block_on(async move {
                let components = merge_components(&config, &instance.config.components)
                    .await
                    .unwrap();

                let prepared =
                    prepare_launch(&config, instance, &components, LaunchOptions::default())
                        .await
                        .unwrap();

                prepared.launch(true).await.unwrap();
            });
        });
    }

    fn create_instance(
        &mut self,
        name: String,
        version: String,
        modloader_string: String,
        modloader_version: String,
    ) {
        let config = Config::new("dev.helixlauncher.HelixLauncher", "HelixLauncher").unwrap();

        let modloader = match &*modloader_string {
            "Quilt" => Modloader::Quilt,
            "Fabric" => Modloader::Fabric,
            "Forge" => Modloader::Forge,
            "" => Modloader::Vanilla,
            _ => unreachable!(),
        };

        Instance::new(
            name,
            version,
            InstanceLaunch::default(),
            &config.get_instances_path(),
            modloader,
            Some(modloader_version),
        )
        .unwrap();

        self.begin_reset_model();
        self.end_reset_model();
    }
}

impl QAbstractListModel for InstancesModel {
    fn row_count(&self) -> i32 {
        let config = Config::new("dev.helixlauncher.HelixLauncher", "HelixLauncher").unwrap();
        let instances = Instance::list_instances(config.get_instances_path()).unwrap();
        instances.len() as _
    }

    fn data(&self, index: QModelIndex, role: i32) -> QVariant {
        let config = Config::new("dev.helixlauncher.HelixLauncher", "HelixLauncher").unwrap();
        let mut instances = Instance::list_instances(config.get_instances_path()).unwrap();
        instances.sort_by(|x, y| x.path.cmp(&y.path));

        if let Some(elem) = instances.get(index.row() as usize) {
            if role == USER_ROLE {
                QString::from(&elem.config.name[..]).into()
            } else if role == USER_ROLE + 1 {
                for component in &elem.config.components {
                    if component.id == "net.fabricmc.fabric-loader" {
                        return QString::from("Fabric").into();
                    } else if component.id == "org.quiltmc.quilt-loader" {
                        return QString::from("Quilt").into();
                    } else if component.id == "net.minecraftforge.forge" {
                        return QString::from("Forge").into();
                    }
                }

                QVariant::default()
            } else if role == USER_ROLE + 2 {
                let minecraft = elem
                    .config
                    .components
                    .iter()
                    .find(|x| x.id == "net.minecraft");
                if let Some(minecraft) = minecraft {
                    QString::from(&minecraft.version[..]).into()
                } else {
                    QVariant::default()
                }
            } else {
                QVariant::default()
            }
        } else {
            QVariant::default()
        }
    }

    fn role_names(&self) -> HashMap<i32, QByteArray> {
        let mut map = HashMap::new();
        map.insert(USER_ROLE, "name".into());
        map.insert(USER_ROLE + 1, "loader".into());
        map.insert(USER_ROLE + 2, "version".into());
        map
    }
}

impl QSingletonInit for InstancesModel {
    fn init(&mut self) {}
}
