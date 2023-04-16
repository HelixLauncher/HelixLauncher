use cstr::cstr;
use helixlauncher_core::config::Config;
use helixlauncher_core::game::{merge_components, prepare_launch, LaunchOptions};
use helixlauncher_core::instance::Instance;
use helixlauncher_core::launcher::launch;
use qmetaobject::prelude::*;
use qmetaobject::qtquickcontrols2::QQuickStyle;
use qmetaobject::USER_ROLE;
use std::collections::HashMap;
use std::env;
use tokio::runtime::Runtime;

#[derive(Default, QObject)]
pub struct InstancesModel {
    base: qt_base_class!(trait QAbstractListModel),

    launch: qt_method!(fn(&self, item: usize)),
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

                launch(&prepared, true).await.unwrap();
            });
        });
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

qrc!(register_resources,
     "qml" as "helixlauncher-gui/qml" {
         "main.qml",
     },
);

fn main() {
    if env::var_os("QT_QUICK_CONTROLS_STYLE").is_none() {
        QQuickStyle::set_style("org.kde.desktop");
    }

    register_resources();

    qml_register_type::<InstancesModel>(
        cstr!("dev.helixlauncher.qml"),
        1,
        0,
        cstr!("InstancesModel"),
    );
    let mut engine = QmlEngine::new();
    engine.load_file("qrc:/helixlauncher-gui/qml/main.qml".into());
    engine.exec();
}
