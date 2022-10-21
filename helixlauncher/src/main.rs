use qmetaobject::prelude::*;
use qmetaobject::qtquickcontrols2::QQuickStyle;
use std::env;

qrc!(register_resources,
     "qml" as "helixlauncher/qml" {
         "main.qml"
     },
);

fn main() {
    register_resources();

    if env::var_os("QT_QUICK_CONTROLS_STYLE").is_none() {
        QQuickStyle::set_style("Imagine");
    }

    let mut engine = QmlEngine::new();
    engine.load_file("qrc:/helixlauncher/qml/main.qml".into());
    engine.exec();
}
