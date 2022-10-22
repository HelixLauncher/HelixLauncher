import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import org.kde.kirigami 2.13 as Kirigami

Kirigami.ApplicationWindow {
    id: root

    title: "Helix Launcher"

    pageStack.globalToolBar.style: pageStack.wideMode ? Kirigami.ApplicationHeaderStyle.None : Kirigami.ApplicationHeaderStyle.Auto

    pageStack.defaultColumnWidth: 200
    pageStack.items: [
        Kirigami.Page {
            Rectangle {
                anchors.fill: parent

                Label { text: "Sidebar"; anchors.centerIn: parent }

                color: "red"
            }
        },

        Kirigami.ScrollablePage {
            Rectangle {
                width: parent.width
                height: 99999

                Label { text: "Instances" }

                color: "green"
            }

            footer: Rectangle {
                width: parent.width
                height: 100

                Label { text: "Footer" }

                color: "blue"
            }
        }
    ]
}
