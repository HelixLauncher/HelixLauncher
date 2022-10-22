import QtQuick 2.15
import QtQuick.Controls 2.15 as Controls
import QtQuick.Layouts 1.15
import org.kde.kirigami 2.13 as Kirigami

Kirigami.AbstractApplicationWindow {
    id: root

    title: "Helix Launcher"

    Kirigami.PageRow {
        anchors.fill: parent

        defaultColumnWidth: 200

        items: [
            Kirigami.Page {
                Rectangle {
                    anchors.fill: parent

                    Controls.Label { text: "Sidebar"; anchors.centerIn: parent }

                    color: "red"
                }
            },

            Kirigami.ScrollablePage {
                Rectangle {
                    width: parent.width
                    height: 99999

                    Controls.Label { text: "Instances" }

                    color: "green"
                }

                footer: Rectangle {
                    width: parent.width
                    height: 100

                    Controls.Label { text: "Footer" }

                    color: "blue"
                }
            }
        ]
    }
}
