import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

ApplicationWindow {
    id: window

    title: "HelixLauncher"
    visible: true

    GridLayout {
        anchors.fill: parent

        rows: 2
        columns: 3

        Rectangle {
            Label { text: "Sidebar"; anchors.centerIn: parent }

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
            Label { text: "Instances"; anchors.centerIn: parent }

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
            Label { text: "Settings"; anchors.centerIn: parent }

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
            Label { text: "Selected"; anchors.centerIn: parent }

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
