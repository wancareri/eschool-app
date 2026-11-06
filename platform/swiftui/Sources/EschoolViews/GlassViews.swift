import SwiftUI

// MARK: - Liquid Glass Header

public struct GlassHeaderView: View {
    let title: String
    let subtitle: String

    public init(title: String, subtitle: String) {
        self.title = title
        self.subtitle = subtitle
    }

    @ViewBuilder
    public var body: some View {
        if #available(iOS 26.0, macOS 26.0, *) {
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
            .glassEffect(.regular.interactive(), in: .rect(cornerRadius: 20))
        } else {
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

    @ViewBuilder
    public var body: some View {
        if #available(iOS 26.0, macOS 26.0, *) {
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
            .glassEffect(.regular, in: .rect(cornerRadius: 16))
        } else {
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
}

// MARK: - Liquid Glass Button

public struct GlassButtonView: View {
    let title: String
    let icon: String

    public init(title: String, icon: String) {
        self.title = title
        self.icon = icon
    }

    @ViewBuilder
    public var body: some View {
        if #available(iOS 26.0, macOS 26.0, *) {
            HStack {
                Image(systemName: icon)
                Text(title)
            }
            .font(.headline)
            .foregroundStyle(.white)
            .frame(maxWidth: .infinity)
            .padding(.vertical, 14)
            .glassEffect(.regular.interactive(), in: .capsule)
        } else {
            HStack {
                Image(systemName: icon)
                Text(title)
            }
            .font(.headline)
            .foregroundStyle(.white)
            .frame(maxWidth: .infinity)
            .padding(.vertical, 14)
            .background(.ultraThinMaterial, in: Capsule())
        }
    }
}

// MARK: - Liquid Glass Status Capsule

public struct GlassStatusCapsuleView: View {
    let title: String
    let icon: String

    public init(title: String, icon: String) {
        self.title = title
        self.icon = icon
    }

    private var iconColor: Color {
        switch icon {
        case "wifi":
            return .accentColor
        case "wifi.slash":
            return .secondary
        case "wifi.exclamationmark":
            return .red
        default:
            return .primary
        }
    }

    @ViewBuilder
    public var body: some View {
        let isExpanded = !title.isEmpty
        HStack(spacing: 5) {
            if icon == "spinner" {
                ProgressView()
                    .progressViewStyle(CircularProgressViewStyle())
                    .tint(.accentColor)
                    .scaleEffect(0.65)
                    .frame(width: 13, height: 13)
            } else if !icon.isEmpty {
                Image(systemName: icon)
                    .font(.system(size: 13, weight: .semibold))
                    .foregroundStyle(iconColor)
                    .frame(width: 14, height: 14)
            }
            if isExpanded {
                Text(title)
                    .font(.system(size: 12.5, weight: .medium))
                    .foregroundStyle(.primary)
                    .lineLimit(1)
                    .transition(
                        .asymmetric(
                            insertion: .opacity
                                .combined(with: .scale(scale: 0.75, anchor: .leading))
                                .combined(with: .offset(x: -3)),
                            removal: .opacity
                                .combined(with: .scale(scale: 0.75, anchor: .leading))
                        )
                    )
            }
        }
        .padding(.horizontal, isExpanded ? 11 : 8.5)
        .frame(height: 30)
        .background(.ultraThinMaterial, in: Capsule())
        .overlay(
            Capsule()
                .stroke(Color.primary.opacity(0.08), lineWidth: 0.5)
        )
        .contentShape(Capsule())
        .animation(.spring(response: 0.38, dampingFraction: 0.72), value: isExpanded)
        .animation(.easeInOut(duration: 0.2), value: icon)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
    }
}

// MARK: - Liquid Glass Navigation Bar

public struct GlassNavBarView: View {
    let title: String

    public init(title: String) {
        self.title = title
    }

    @ViewBuilder
    public var body: some View {
        if #available(iOS 26.0, macOS 26.0, *) {
            Text(title)
                .font(.headline)
                .frame(maxWidth: .infinity)
                .padding(.vertical, 12)
                .glassEffect(.regular, in: .rect(cornerRadius: 12))
        } else {
            Text(title)
                .font(.headline)
                .frame(maxWidth: .infinity)
                .padding(.vertical, 12)
                .background(.ultraThinMaterial, in: RoundedRectangle(cornerRadius: 12))
        }
    }
}

// MARK: - Native Quick Look & File Share

#if canImport(UIKit)
import UIKit

// MARK: - True Native Liquid Glass Button Styler

final class LiquidGlassContainerView: UIView {
    let blurView: UIVisualEffectView
    let tintView: UIView
    let gradientLayer: CAGradientLayer
    let shapeLayer: CAShapeLayer

    init(size: CGFloat) {
        let frame = CGRect(x: 0, y: 0, width: size, height: size)
        let radius = size / 2.0

        let blur = UIBlurEffect(style: .systemUltraThinMaterial)
        self.blurView = UIVisualEffectView(effect: blur)
        self.blurView.frame = frame
        self.blurView.layer.cornerRadius = radius
        self.blurView.layer.masksToBounds = true

        self.tintView = UIView(frame: frame)
        self.tintView.layer.cornerRadius = radius
        self.tintView.layer.masksToBounds = true

        self.gradientLayer = CAGradientLayer()
        self.gradientLayer.frame = frame
        self.gradientLayer.startPoint = CGPoint(x: 0.0, y: 0.0)
        self.gradientLayer.endPoint = CGPoint(x: 1.0, y: 1.0)

        self.shapeLayer = CAShapeLayer()
        self.shapeLayer.lineWidth = 1.2
        let path = UIBezierPath(ovalIn: frame.insetBy(dx: 0.6, dy: 0.6))
        self.shapeLayer.path = path.cgPath
        self.shapeLayer.strokeColor = UIColor.black.cgColor
        self.shapeLayer.fillColor = UIColor.clear.cgColor
        self.gradientLayer.mask = self.shapeLayer

        super.init(frame: frame)
        self.isUserInteractionEnabled = false
        self.autoresizingMask = [.flexibleWidth, .flexibleHeight]

        addSubview(blurView)
        addSubview(tintView)
        layer.addSublayer(gradientLayer)

        updateAppearance()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func layoutSubviews() {
        super.layoutSubviews()
        let radius = bounds.width / 2.0
        blurView.frame = bounds
        blurView.layer.cornerRadius = radius
        tintView.frame = bounds
        tintView.layer.cornerRadius = radius
        gradientLayer.frame = bounds
        let path = UIBezierPath(ovalIn: bounds.insetBy(dx: 0.6, dy: 0.6))
        shapeLayer.path = path.cgPath
    }

    func updateAppearance() {
        let isDark = traitCollection.userInterfaceStyle == .dark
        tintView.backgroundColor = isDark ? UIColor(white: 1.0, alpha: 0.14) : UIColor(white: 1.0, alpha: 0.52)

        gradientLayer.colors = [
            (isDark ? UIColor(white: 1.0, alpha: 0.80) : UIColor(white: 1.0, alpha: 0.95)).cgColor,
            (isDark ? UIColor(white: 1.0, alpha: 0.18) : UIColor(white: 1.0, alpha: 0.35)).cgColor,
            (isDark ? UIColor(white: 1.0, alpha: 0.06) : UIColor(white: 1.0, alpha: 0.12)).cgColor,
            (isDark ? UIColor(white: 1.0, alpha: 0.45) : UIColor(white: 1.0, alpha: 0.60)).cgColor
        ]
    }

    override func traitCollectionDidChange(_ previousTraitCollection: UITraitCollection?) {
        super.traitCollectionDidChange(previousTraitCollection)
        updateAppearance()
    }
}

@_cdecl("eschool_style_liquid_glass_button")
public func eschool_style_liquid_glass_button(buttonPtr: UnsafeMutableRawPointer) {
    let raw = Unmanaged<UIView>.fromOpaque(buttonPtr).takeUnretainedValue()
    let btn: UIButton = (raw as? UIButton) ?? raw.subviews.compactMap({ $0 as? UIButton }).first ?? {
        let b = UIButton(frame: raw.bounds)
        raw.addSubview(b)
        return b
    }()

    let size: CGFloat = 42.0
    let radius: CGFloat = size / 2.0

    if btn.bounds.width == 0 || btn.bounds.height == 0 {
        btn.bounds = CGRect(x: 0, y: 0, width: size, height: size)
    }

    btn.setTitle(nil, for: .normal)
    btn.backgroundColor = .clear

    let tag = 99881
    if let existing = btn.viewWithTag(tag) {
        existing.removeFromSuperview()
    }

    let glassView = LiquidGlassContainerView(size: size)
    glassView.tag = tag

    // Drop Shadow on the button
    btn.layer.cornerRadius = radius
    btn.layer.shadowColor = UIColor.black.cgColor
    btn.layer.shadowOpacity = 0.16
    btn.layer.shadowRadius = 8.0
    btn.layer.shadowOffset = CGSize(width: 0.0, height: 3.0)
    btn.layer.masksToBounds = false

    btn.insertSubview(glassView, at: 0)

    // Centered SF Symbol chevron with semibold weight
    let config = UIImage.SymbolConfiguration(pointSize: 17.0, weight: .semibold)
    let img = UIImage(systemName: "chevron.backward", withConfiguration: config)
    btn.setImage(img, for: .normal)
    btn.tintColor = .label
    btn.contentHorizontalAlignment = .center
    btn.contentVerticalAlignment = .center
}

// MARK: - Hardware-Synchronized Keyboard Button Gluer

final class KeyboardGluer: NSObject {
    static let shared = KeyboardGluer()
    private weak var button: UIButton?

    override init() {
        super.init()
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(keyboardWillChange(_:)),
            name: UIResponder.keyboardWillChangeFrameNotification,
            object: nil
        )
    }

    func register(button: UIButton) {
        self.button = button
    }

    @objc private func keyboardWillChange(_ notification: Notification) {
        guard let button = self.button,
              let userInfo = notification.userInfo,
              let endFrame = (userInfo[UIResponder.keyboardFrameEndUserInfoKey] as? NSValue)?.cgRectValue,
              let duration = (userInfo[UIResponder.keyboardAnimationDurationUserInfoKey] as? NSNumber)?.doubleValue,
              let curveValue = (userInfo[UIResponder.keyboardAnimationCurveUserInfoKey] as? NSNumber)?.intValue
        else { return }

        guard let window = button.window ?? UIApplication.shared.connectedScenes
            .compactMap({ $0 as? UIWindowScene })
            .flatMap({ $0.windows })
            .first(where: { $0.isKeyWindow })
        else { return }

        let screenHeight = window.bounds.height
        let safeAreaBottom = window.safeAreaInsets.bottom

        let keyboardHeight: CGFloat = endFrame.origin.y < screenHeight ? (screenHeight - endFrame.origin.y) : 0
        let lift: CGFloat = keyboardHeight > 0 ? max(0, keyboardHeight - safeAreaBottom) : 0

        let options = UIView.AnimationOptions(rawValue: UInt(curveValue << 16))

        UIView.animate(
            withDuration: duration,
            delay: 0,
            options: [options, .beginFromCurrentState, .allowUserInteraction],
            animations: {
                button.transform = lift > 0 ? CGAffineTransform(translationX: 0, y: -lift) : .identity
            },
            completion: nil
        )
    }
}

@_cdecl("eschool_glue_button_to_keyboard")
public func eschool_glue_button_to_keyboard(buttonPtr: UnsafeMutableRawPointer) {
    let raw = Unmanaged<UIView>.fromOpaque(buttonPtr).takeUnretainedValue()
    let btn: UIButton? = (raw as? UIButton) ?? raw.subviews.compactMap({ $0 as? UIButton }).first
    if let b = btn {
        KeyboardGluer.shared.register(button: b)
    }
}
#if canImport(QuickLook)
import QuickLook

public final class EschoolPreviewItem: NSObject, QLPreviewItem {
    public let previewItemURL: URL?
    public let previewItemTitle: String?

    public init(url: URL, title: String? = nil) {
        self.previewItemURL = url
        self.previewItemTitle = title ?? url.lastPathComponent
        super.init()
    }
}

public final class EschoolQuickLookDataSource: NSObject, QLPreviewControllerDataSource {
    let item: EschoolPreviewItem

    public init(item: EschoolPreviewItem) {
        self.item = item
        super.init()
    }

    public func numberOfPreviewItems(in controller: QLPreviewController) -> Int {
        return 1
    }

    public func previewController(_ controller: QLPreviewController, previewItemAt index: Int) -> QLPreviewItem {
        return item
    }
}

private var qlDataSourceKey: UInt8 = 0
#endif

@_cdecl("eschool_quicklook_file")
public func eschool_quicklook_file(path: UnsafePointer<CChar>) {
    let filePath = String(cString: path)
    DispatchQueue.main.async {
        guard let windowScene = UIApplication.shared.connectedScenes.compactMap({ $0 as? UIWindowScene }).first,
              let rootVC = windowScene.windows.first(where: { $0.isKeyWindow })?.rootViewController else {
            return
        }

        var topVC = rootVC
        while let presented = topVC.presentedViewController {
            topVC = presented
        }

        let url = URL(fileURLWithPath: filePath)
        #if canImport(QuickLook)
        let previewItem = EschoolPreviewItem(url: url)
        let dataSource = EschoolQuickLookDataSource(item: previewItem)

        let qlController = QLPreviewController()
        qlController.dataSource = dataSource
        objc_setAssociatedObject(qlController, &qlDataSourceKey, dataSource, .OBJC_ASSOCIATION_RETAIN_NONATOMIC)

        topVC.present(qlController, animated: true)
        #else
        let activityVC = UIActivityViewController(activityItems: [url], applicationActivities: nil)
        if let popover = activityVC.popoverPresentationController {
            popover.sourceView = topVC.view
        }
        topVC.present(activityVC, animated: true)
        #endif
    }
}

@_cdecl("eschool_share_file")
public func eschool_share_file(path: UnsafePointer<CChar>) {
    let filePath = String(cString: path)
    DispatchQueue.main.async {
        guard let windowScene = UIApplication.shared.connectedScenes.compactMap({ $0 as? UIWindowScene }).first,
              let rootVC = windowScene.windows.first(where: { $0.isKeyWindow })?.rootViewController else {
            return
        }

        var topVC = rootVC
        while let presented = topVC.presentedViewController {
            topVC = presented
        }

        let url = URL(fileURLWithPath: filePath)
        let activityVC = UIActivityViewController(activityItems: [url], applicationActivities: nil)
        if let popover = activityVC.popoverPresentationController {
            popover.sourceView = topVC.view
        }
        topVC.present(activityVC, animated: true)
    }
}

@_cdecl("eschool_open_url")
public func eschool_open_url(urlStr: UnsafePointer<CChar>) {
    let s = String(cString: urlStr)
    if let url = URL(string: s) {
        DispatchQueue.main.async {
            UIApplication.shared.open(url)
        }
    }
}
#elseif os(macOS)
import AppKit

@_cdecl("eschool_quicklook_file")
public func eschool_quicklook_file(path: UnsafePointer<CChar>) {
    let filePath = String(cString: path)
    DispatchQueue.main.async {
        NSWorkspace.shared.open(URL(fileURLWithPath: filePath))
    }
}

@_cdecl("eschool_share_file")
public func eschool_share_file(path: UnsafePointer<CChar>) {
    let filePath = String(cString: path)
    DispatchQueue.main.async {
        NSWorkspace.shared.open(URL(fileURLWithPath: filePath))
    }
}

@_cdecl("eschool_open_url")
public func eschool_open_url(urlStr: UnsafePointer<CChar>) {
    let s = String(cString: urlStr)
    if let url = URL(string: s) {
        DispatchQueue.main.async {
            NSWorkspace.shared.open(url)
        }
    }
}
#endif

