import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import dev.helixlauncher.qml 1.0
import org.kde.kirigami 2.13 as Kirigami

Kirigami.Page {
    //SignInModel.start_auth()

    Component.onCompleted: {
        console.log("hi");
    }

    ColumnLayout {
        spacing: 0

        anchors {
            verticalCenter: parent.verticalCenter
            right: parent.right
            left: parent.left
        }

        RowLayout {
            TextEdit {
                text: SignInModel.get_message()
                readOnly: true
                selectByMouse: true
            }

        }

        RowLayout {
            Layout.fillWidth: true
            Layout.topMargin: Kirigami.Units.smallSpacing

            Button {
                text: "Open page"
                onClicked: {
                    Qt.openUrlExternally("https://microsoft.com/link")
                }
            }

        }

    }

}
