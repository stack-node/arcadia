import Foundation

struct PageDefinition: Identifiable, Decodable {
    let id: String
    let title: String
    let description: String
    let glyph: String
    let systemImage: String

    enum CodingKeys: String, CodingKey {
        case id
        case title
        case description
        case glyph
        case systemImage = "system_image"
    }
}

struct GroupDefinition: Identifiable, Decodable {
    let id: String
    let label: String
    let glyph: String
    let systemImage: String
    let pageIDs: [String]

    enum CodingKeys: String, CodingKey {
        case id
        case label
        case glyph
        case systemImage = "system_image"
        case pageIDs = "pages"
    }
}

struct NavigationRegistry: Decodable {
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
