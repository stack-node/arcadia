import SwiftUI

struct SidebarView: View {
    @Environment(\.colorScheme) private var colorScheme
    private var theme: AppTheme { AppTheme(isDark: colorScheme == .dark) }

    let registry: NavigationRegistry
    let sidebarWidth: CGFloat
    let sidebarSwipeThreshold: CGFloat
    let shellEnabled: Bool
    let netEnabled: Bool
    let remoteSessionEnabled: Bool

    @Binding var activeGroupID: String
    @Binding var activePageID: String

    private var visibleGroups: [GroupDefinition] {
        registry.groups.filter { group in
            group.pageIDs.contains { isPageVisible($0) }
        }
    }

    private var activeGroup: GroupDefinition {
        visibleGroups.first(where: { $0.id == activeGroupID })
            ?? visibleGroups.first
            ?? registry.groups[0]
    }

    private var activeGroupPages: [PageDefinition] {
        activeGroup.pageIDs
            .filter { isPageVisible($0) }
            .compactMap { id in registry.pages.first(where: { $0.id == id }) }
    }

    private func isPageVisible(_ pageID: String) -> Bool {
        switch pageID {
        case "utility.shell":
            return shellEnabled
        case "network.overview":
            return netEnabled
        default:
            return true
        }
    }

    private func selectGroup(_ groupID: String) {
        activeGroupID = groupID
        if let group = visibleGroups.first(where: { $0.id == groupID }),
           let firstPageID = group.pageIDs.first(where: { isPageVisible($0) }) {
            activePageID = firstPageID
        }
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            VStack(alignment: .leading, spacing: 6) {
                Text("Arcadia")
                    .font(.system(size: 28, weight: .semibold, design: .rounded))
                    .foregroundStyle(theme.primaryTextColor)

                if remoteSessionEnabled {
                    Menu {
                        Button("local") {}
                    } label: {
                        HStack(spacing: 6) {
                            Text("local")
                            Image(systemName: "chevron.down")
                                .font(.system(size: 10, weight: .semibold))
                        }
                        .font(.caption.weight(.medium))
                        .foregroundStyle(theme.secondaryTextColor)
                        .padding(.horizontal, 10)
                        .padding(.vertical, 6)
                        .background(theme.cardFillColor, in: Capsule())
                        .overlay {
                            Capsule()
                                .stroke(theme.cardStrokeColor, lineWidth: 1)
                        }
                    }
                }

                Text("Liquid glass")
                    .font(.subheadline)
                    .foregroundStyle(theme.secondaryTextColor)
            }
            .padding(.horizontal, 22)
            .padding(.top, 28)
            .padding(.bottom, 10)

            Text("Groups")
                .font(.caption.weight(.semibold))
                .foregroundStyle(theme.tertiaryTextColor)
                .padding(.horizontal, 16)

            ScrollView(.horizontal, showsIndicators: false) {
                HStack(spacing: 8) {
                    ForEach(visibleGroups) { group in
                        Button {
                            selectGroup(group.id)
                        } label: {
                            VStack(spacing: 6) {
                                Image(systemName: group.systemImage)
                                    .font(.system(size: 14, weight: .semibold))
                                Text(group.label)
                            }
                            .font(.caption.weight(activeGroupID == group.id ? .semibold : .medium))
                            .foregroundStyle(activeGroupID == group.id ? theme.selectedTextColor : theme.secondaryTextColor)
                            .frame(width: 64, height: 64)
                            .background(
                                RoundedRectangle(cornerRadius: 12, style: .continuous)
                                    .fill(activeGroupID == group.id ? theme.selectedFillColor : .clear)
                            )
                            .overlay {
                                RoundedRectangle(cornerRadius: 12, style: .continuous)
                                    .stroke(activeGroupID == group.id ? theme.selectedStrokeColor : .clear, lineWidth: 1)
                            }
                        }
                        .buttonStyle(.plain)
                    }
                }
                .padding(.horizontal, 14)
            }

            ScrollView(.vertical, showsIndicators: false) {
                VStack(alignment: .leading, spacing: 8) {
                    Text(activeGroup.label)
                        .font(.caption.weight(.semibold))
                        .foregroundStyle(theme.tertiaryTextColor)
                        .padding(.top, 2)
                        .padding(.horizontal, 16)

                    ForEach(activeGroupPages) { page in
                        pageButton(page: page)
                    }
                }
                .padding(.bottom, 8)
            }
            .frame(maxHeight: .infinity)

            Text("Global")
                .font(.caption.weight(.semibold))
                .foregroundStyle(theme.tertiaryTextColor)
                .padding(.horizontal, 16)

            ForEach(registry.globalPages, id: \.self) { pageID in
                if let page = registry.pages.first(where: { $0.id == pageID }) {
                    pageButton(page: page)
                }
            }
            .padding(.bottom, 14)
        }
        .frame(width: sidebarWidth)
        .frame(maxHeight: .infinity, alignment: .topLeading)
        .background(.ultraThinMaterial)
        .background(colorScheme == .dark ? .white.opacity(0.05) : .white.opacity(0.3))
        .overlay {
            RoundedRectangle(cornerRadius: 0)
                .stroke(theme.cardStrokeColor, lineWidth: 1)
        }
        .shadow(color: theme.sidebarShadowColor, radius: 28, x: 8, y: 0)
        .ignoresSafeArea()
    }

    private func pageButton(page: PageDefinition) -> some View {
        let isActive = activePageID == page.id
        return Button {
            activePageID = page.id
        } label: {
            HStack(spacing: 12) {
                Image(systemName: page.systemImage)
                    .font(.system(size: 16, weight: .semibold))
                    .frame(width: 20)
                Text(page.title)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
            .font(.body.weight(isActive ? .semibold : .medium))
            .foregroundStyle(isActive ? theme.selectedTextColor : theme.secondaryTextColor)
            .padding(.horizontal, 16)
            .frame(height: 50)
            .background(
                RoundedRectangle(cornerRadius: 16, style: .continuous)
                    .fill(isActive ? theme.selectedFillColor : .clear)
            )
            .overlay {
                RoundedRectangle(cornerRadius: 16, style: .continuous)
                    .stroke(isActive ? theme.selectedStrokeColor : .clear, lineWidth: 1)
            }
        }
        .buttonStyle(.plain)
        .padding(.horizontal, 14)
    }
}
