use qmetaobject::prelude::*;
use qmetaobject::qtquickcontrols2::QQuickStyle;
use std::env;

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

    let mut engine = QmlEngine::new();
    engine.load_file("qrc:/helixlauncher-gui/qml/main.qml".into());
    engine.exec();
}
