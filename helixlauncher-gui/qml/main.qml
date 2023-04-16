import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import org.kde.kirigami 2.13 as Kirigami
import dev.helixlauncher.qml 1.0

Kirigami.ApplicationWindow {
    id: root

    title: "Helix Launcher"

    pageStack.items: [
        Kirigami.ScrollablePage {
            title: "Instances"

            actions.main: Kirigami.Action {
                icon.name: "list-add"
                text: "Add instance"
            }

            Kirigami.CardsListView {
                model: InstancesModel {
                    id: instancesModel
                }

                delegate: Kirigami.AbstractCard {
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

                    showClickFeedback: true

                    onClicked: {
                        applicationWindow().pageStack.push(instancePage)
                    }

                    Component {
                        id: instancePage
                        Kirigami.Page {
                            title: name

                            Kirigami.Heading {
                                level: 1
                                text: name
                            }

                            actions.main: Kirigami.Action {
                                text: "Launch"
                                icon.name: "media-playback-start"
                                onTriggered: instancesModel.launch(index)
                            }

                            Shortcut {
                                sequences: [ StandardKey.Cancel ]
                                enabled: isCurrentPage && applicationWindow().pageStack.depth > 1
                                onActivated: {
                                    applicationWindow().pageStack.pop()
                                }
                            }
                        }
                    }
                }
            }
        }
    ]
}
