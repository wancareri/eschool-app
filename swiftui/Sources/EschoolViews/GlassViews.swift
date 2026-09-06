import SwiftUI

// MARK: - Liquid Glass Header
// Uses .ultraThinMaterial for translucent glass-like appearance.
// When building with Xcode 16+/iOS 26+ SDK, replace with .glassEffect(.regular, in: ...).

public struct GlassHeaderView: View {
    let title: String
    let subtitle: String

    public init(title: String, subtitle: String) {
        self.title = title
        self.subtitle = subtitle
    }

    public var body: some View {
        VStack(spacing: 12) {
            Text(title)
                .font(.largeTitle.bold())
                .foregroundStyle(.primary)
            Text(subtitle)
                .font(.subheadline)
                .foregroundStyle(.secondary)
        }
        .frame(maxWidth: .infinity)
        .padding(.vertical, 24)
        .background(.ultraThinMaterial, in: RoundedRectangle(cornerRadius: 20))
    }
}

// MARK: - Liquid Glass Card

public struct GlassCardView: View {
    let title: String
    let value: String
    let icon: String

    public init(title: String, value: String, icon: String) {
        self.title = title
        self.value = value
        self.icon = icon
    }

    public var body: some View {
        VStack(spacing: 8) {
            Image(systemName: icon)
                .font(.title2)
                .foregroundStyle(.blue)
            Text(value)
                .font(.title.bold())
            Text(title)
                .font(.caption)
                .foregroundStyle(.secondary)
        }
        .frame(maxWidth: .infinity)
        .padding(.vertical, 16)
        .background(.ultraThinMaterial, in: RoundedRectangle(cornerRadius: 16))
    }
}

// MARK: - Liquid Glass Button

public struct GlassButtonView: View {
    let title: String
    let icon: String

    public init(title: String, icon: String) {
        self.title = title
        self.icon = icon
    }

    public var body: some View {
        HStack {
            Image(systemName: icon)
            Text(title)
        }
        .font(.headline)
        .foregroundStyle(.white)
        .frame(maxWidth: .infinity)
        .padding(.vertical, 14)
        .background(.regularMaterial, in: Capsule())
    }
}

// MARK: - Liquid Glass Navigation Bar

public struct GlassNavBarView: View {
    let title: String

    public init(title: String) {
        self.title = title
    }

    public var body: some View {
        Text(title)
            .font(.headline)
            .frame(maxWidth: .infinity)
            .padding(.vertical, 12)
            .background(.ultraThinMaterial, in: RoundedRectangle(cornerRadius: 12))
    }
}
