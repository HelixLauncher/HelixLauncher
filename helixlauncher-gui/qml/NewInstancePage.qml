import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import org.kde.kirigami 2.13 as Kirigami
import dev.helixlauncher.qml 1.0

Kirigami.Page {
    id: root

    title: "New Instance"

    Kirigami.FormLayout {
        anchors.fill: parent

        TextField {
            id: instanceName
            Kirigami.FormData.label: "Name:"
        }

        TextField {
            id: instanceVersion
            Kirigami.FormData.label: "Version:"
        }

        ComboBox {
            id: instanceLoader
            Kirigami.FormData.label: "Loader:"
            Kirigami.FormData.checkable: true
            enabled: Kirigami.FormData.checked

            model: ["Quilt", "Fabric", "Forge"]
        }

        TextField {
            id: instanceLoaderVersion
            Kirigami.FormData.label: "Loader version:"
            enabled: instanceLoader.Kirigami.FormData.checked
        }
    }

    footer: DialogButtonBox {
        standardButtons: DialogButtonBox.Ok | DialogButtonBox.Cancel
        position: DialogButtonBox.Footer

        onAccepted: {
            InstancesModel.create_instance(
                instanceName.text,
                instanceVersion.text,
                instanceLoader.Kirigami.FormData.checked ? instanceLoader.currentText : "",
                instanceLoaderVersion.text
            )
            root.closeDialog()
        }

        onRejected: root.closeDialog()
    }
}
