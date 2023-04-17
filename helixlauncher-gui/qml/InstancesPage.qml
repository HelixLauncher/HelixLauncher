import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import org.kde.kirigami 2.13 as Kirigami
import dev.helixlauncher.qml 1.0

Kirigami.ScrollablePage {
    title: "Instances"

    InstancesModel {
        id: instancesModel
    }

    actions.main: Kirigami.Action {
        icon.name: "list-add"
        text: "Add instance"
        onTriggered: applicationWindow().pageStack.pushDialogLayer("qrc:/qml/NewInstancePage.qml", { instancesModel }, {
            title: "New Instance",
            width: Kirigami.Units.gridUnit * 20,
            height: Kirigami.Units.gridUnit * 10
        })
    }

    Kirigami.CardsListView {
        model: instancesModel

        delegate: Kirigami.AbstractCard {
            showClickFeedback: true

            onClicked: {
                applicationWindow().pageStack.push('qrc:/qml/InstancePage.qml', { name, index, instancesModel })
            }

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
                            text: name
                            elide: Text.ElideRight
                            maximumLineCount: 1
                        }

                        RowLayout {
                            Layout.alignment: Qt.AlignRight

                            Label {
                                text: loader
                                font: Kirigami.Theme.smallFont
                            }

                            Label {
                                text: version
                                font: Kirigami.Theme.smallFont
                            }
                        }
                    }

                    RowLayout {
                        Layout.fillWidth: true
                        Layout.topMargin: Kirigami.Units.smallSpacing

                        Label {
                            Layout.fillWidth: true
                            Layout.alignment: Qt.AlignTop
                            text: "Playtime unknown"
                            opacity: 0.6
                            font: Kirigami.Theme.smallFont
                            elide: Text.ElideRight
                        }

                        Button {
                            Layout.alignment: Qt.AlignVCenter | Qt.AlignRight
                            text: "Launch"
                            onClicked: instancesModel.launch(index)
                        }
                    }
                }
            }
        }
    }
}
