import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import org.kde.kirigami 2.13 as Kirigami

Kirigami.ApplicationWindow {
    id: root

    title: "Helix Launcher"

    pageStack.initialPage: "qrc:/qml/InstancesPage.qml"
}
