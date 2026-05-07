import Foundation

struct PageDefinition: Identifiable, Codable {
    let id: String
    let title: String
    let description: String
    let glyph: String
    let systemImage: String
    let accent: String
    /// Module registry name; when set, the page is visible only if that module is enabled.
    let requiredModule: String?

    enum CodingKeys: String, CodingKey {
        case id
        case title
        case description
        case glyph
        case systemImage = "system_image"
        case accent
        case requiredModule = "required_module"
    }

    init(
        id: String,
        title: String,
        description: String,
        glyph: String,
        systemImage: String,
        accent: String,
        requiredModule: String? = nil
    ) {
        self.id = id
        self.title = title
        self.description = description
        self.glyph = glyph
        self.systemImage = systemImage
        self.accent = accent
        self.requiredModule = requiredModule
    }
}

struct GroupDefinition: Identifiable, Codable {
    let id: String
    let label: String
    let glyph: String
    let systemImage: String
    let pageIDs: [String]
    let accent: String

    enum CodingKeys: String, CodingKey {
        case id
        case label
        case glyph
        case systemImage = "system_image"
        case pageIDs = "pages"
        case accent
    }
}

struct NavigationRegistry: Codable {
    let pages: [PageDefinition]
    let groups: [GroupDefinition]
    let globalPages: [String]
    let defaultGroup: String
    let defaultPage: String

    enum CodingKeys: String, CodingKey {
        case pages
        case groups
        case globalPages = "global_pages"
        case defaultGroup = "default_group"
        case defaultPage = "default_page"
    }
}
