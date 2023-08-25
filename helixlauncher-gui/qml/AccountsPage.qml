import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import org.kde.kirigami 2.13 as Kirigami
import dev.helixlauncher.qml 1.0

Kirigami.ScrollablePage {
    id: acc_manager_root
    title: "Account Manager"

    actions.main: Kirigami.Action {
        icon.name: "list-add"
        text: "Add Account"
    }

    Kirigami.CardsListView {
        model: AccountsModel

        delegate: Kirigami.AbstractCard {
            showClickFeedback: true

            contentItem: Item {
                implicitHeight: Kirigami.Units.gridUnit * 2

                ColumnLayout {
                    anchors {
                        verticalCenter: parent.verticalCenter
                        right: parent.right
                        left: parent.left
                    }
                    spacing: 0

                    RowLayout {
                        Kirigami.Heading {
                            Layout.fillWidth: true
                            level: 1
                            type: Kirigami.Heading.Type.Primary
                            text: username
                            elide: Text.ElideRight
                            maximumLineCount: 1
                        }

                        RowLayout {
                            Layout.alignment: Qt.AlignRight
                        }
                    }

                    RowLayout {
                        Layout.fillWidth: true
                        Layout.topMargin: Kirigami.Units.smallSpacing

                        Label {
                            Layout.fillWidth: true
                            Layout.alignment: Qt.AlignTop
                            text: "UUID: " + uuid
                            opacity: 0.6
                            font: Kirigami.Theme.smallFont
                            elide: Text.ElideRight
                        }
                        Button {
                            Layout.alignment: Qt.AlignVCenter | Qt.AlignRight
                            text: "Set Default"
                            onClicked: AccountsModel.set_default(uuid)
                        }
                        Button {
                            icon.name: "remove"
                            Layout.alignment: Qt.AlignVCenter | Qt.AlignRight
                            text: "Remove"
                            onClicked: { 
                                AccountsModel.remove(uuid)
                            }
                        }
                    }
                }
            }
        }
    }
}
