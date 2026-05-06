import SwiftUI

extension ContentView {
    func isModuleEnabled(_ name: String) -> Bool {
        modules.first(where: { $0.name == name })?.enabled ?? false
    }

    func isPageVisible(_ pageID: String) -> Bool {
        guard let page = registry.pages.first(where: { $0.id == pageID }) else {
            return false
        }
        guard let required = page.requiredModule, !required.isEmpty else {
            return true
        }
        return isModuleEnabled(required)
    }

    var activePage: PageDefinition {
        if isPageVisible(activePageID), let page = registry.pages.first(where: { $0.id == activePageID }) {
            return page
        }
        if let firstVisible = registry.pages.first(where: { isPageVisible($0.id) }) {
            return firstVisible
        }
        return registry.pages[0]
    }
}
