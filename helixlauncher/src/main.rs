use qmetaobject::prelude::*;

qrc!(register_resources,
     "qml" as "helixlauncher/qml" {
         "main.qml"
     },
);

fn main() {
    register_resources();

    let mut engine = QmlEngine::new();
    engine.load_file("qrc:/helixlauncher/qml/main.qml".into());
    engine.exec();
}
