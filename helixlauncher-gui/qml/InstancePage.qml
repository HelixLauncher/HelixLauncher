import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import org.kde.kirigami 2.13 as Kirigami
import dev.helixlauncher.qml 1.0

Kirigami.Page {
    property string name
    property int index

    title: name

    Kirigami.Heading {
        level: 1
        text: name
    }

    actions.main: Kirigami.Action {
        text: "Launch"
        icon.name: "media-playback-start"
        onTriggered: InstancesModel.launch(index)
    }

    Shortcut {
        sequences: [ StandardKey.Cancel ]
        enabled: isCurrentPage && applicationWindow().pageStack.depth > 1
        onActivated: {
            applicationWindow().pageStack.pop()
        }
    }
}
