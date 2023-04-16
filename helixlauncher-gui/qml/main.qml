import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import org.kde.kirigami 2.13 as Kirigami

Kirigami.ApplicationWindow {
    id: root

    title: "Helix Launcher"

    pageStack.items: [
        Kirigami.ScrollablePage {
            Kirigami.CardsListView {
                model: ListModel {
                    ListElement { name: "Adrenaline AU"; loader: "Quilt"; version: "1.19.2"; }
                    ListElement { name: "Fabulously Optimized"; loader: "Fabric"; version: "1.19.2"; }
                    ListElement { name: "Simply Optimized"; loader: "Fabric"; version: "1.19.2"; }
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
                            text: loader + " " + version
                        }
                    }
                }
            }
        }
    ]
}
