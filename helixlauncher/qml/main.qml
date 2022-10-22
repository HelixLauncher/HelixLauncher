import QtQuick 2.15
import QtQuick.Controls 2.15 as Controls
import QtQuick.Layouts 1.15
import org.kde.kirigami 2.13 as Kirigami

Kirigami.AbstractApplicationWindow {
    id: root

    title: "Helix Launcher"

    Kirigami.Page {
        anchors.fill: parent

        Kirigami.OverlayDrawer {
            id: sidebar

            edge: Qt.LeftEdge
            modal: false

            width: 200

            contentItem: Rectangle {
                Controls.Label { text: "Sidebar"; anchors.centerIn: parent }

                color: "red"

                Layout.fillWidth: true
                Layout.fillHeight: true
            }
        }

        Kirigami.OverlayDrawer {
            id: bottomDrawer

            leftPadding: sidebar.width

            edge: Qt.BottomEdge
            modal: false

            height: 100

            contentItem: Rectangle {
                Controls.Label { text: "Bottom"; anchors.centerIn: parent }

                color: "green"

                Layout.fillWidth: true
                Layout.fillHeight: true
            }
        }

        GridLayout {
            anchors.fill: parent
            anchors.leftMargin: sidebar.width
            anchors.bottomMargin: bottomDrawer.height

            rows: 2
            columns: 3

            Rectangle {
                Controls.Label { text: "Sidebar"; anchors.centerIn: parent }

                color: "red"

                Layout.preferredWidth: 200
                Layout.fillWidth: true
                Layout.fillHeight: true

                Layout.row: 0
                Layout.column: 0
                Layout.rowSpan: 2
                Layout.columnSpan: 1
            }

            Rectangle {
                Controls.Label { text: "Instances"; anchors.centerIn: parent }

                color: "blue"

                Layout.preferredWidth: 1000
                Layout.preferredHeight: 800
                Layout.fillWidth: true
                Layout.fillHeight: true

                Layout.row: 0
                Layout.column: 1
                Layout.rowSpan: 1
                Layout.columnSpan: 2
            }

            Rectangle {
                Controls.Label { text: "Settings"; anchors.centerIn: parent }

                color: "green"

                Layout.minimumWidth: 200
                Layout.preferredHeight: 100
                Layout.fillWidth: true
                Layout.fillHeight: true

                Layout.row: 1
                Layout.column: 1
                Layout.rowSpan: 1
                Layout.columnSpan: 1
            }

            Rectangle {
                Controls.Label { text: "Selected"; anchors.centerIn: parent }

                color: "orange"

                Layout.minimumWidth: 200
                Layout.preferredHeight: 100
                Layout.fillWidth: true
                Layout.fillHeight: true

                Layout.row: 1
                Layout.column: 2
                Layout.rowSpan: 1
                Layout.columnSpan: 1
            }
        }
    }
}
