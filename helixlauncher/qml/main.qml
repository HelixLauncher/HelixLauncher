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
            Kirigami.CardsListView {
                model: ListModel {
                    ListElement { name: "Adrenaline AU"; version: "Quilt 1.19.2"; playTime: "182 hours and 32 minutes" }
                    ListElement { name: "Fabulously Optimized"; version: "Fabric 1.19.2"; playTime: "43 hours and 3 minutes" }
                    ListElement { name: "Simply Optimized"; version: "Fabric 1.19.2"; playTime: "3 hours and 17 minutes" }
                }

                delegate: Kirigami.AbstractCard {
                    contentItem: RowLayout {
                        spacing: Kirigami.Units.gridUnit

                        Kirigami.Heading {
                            Layout.fillWidth: true
                            Layout.fillHeight: true

                            level: 1
                            text: name
                        }

                        Kirigami.Heading {
                            Layout.fillHeight: true

                            level: 3
                            color: Kirigami.Theme.disabledTextColor
                            text: version
                        }

                        Kirigami.Heading {
                            Layout.fillHeight: true

                            level: 3
                            color: Kirigami.Theme.disabledTextColor
                            text: playTime
                        }
                    }
                }
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
