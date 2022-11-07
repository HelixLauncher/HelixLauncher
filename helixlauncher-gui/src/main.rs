use qmetaobject::prelude::*;
use qmetaobject::qtquickcontrols2::QQuickStyle;
use std::env;

qrc!(register_resources,
     "icons" as "helixlauncher-gui/icons" {
         "settings.svg",
     },
     "qml" as "helixlauncher-gui/qml" {
         "main.qml",
     },
);

fn main() {
    if env::var_os("QT_QUICK_CONTROLS_STYLE").is_none() {
        QQuickStyle::set_style("Imagine");
        env::set_var("KIRIGAMI_FORCE_STYLE", "1"); // there's probably a better way to do this
    }

    register_resources();

    let mut engine = QmlEngine::new();
    engine.load_file("qrc:/helixlauncher-gui/qml/main.qml".into());
    engine.exec();
}
