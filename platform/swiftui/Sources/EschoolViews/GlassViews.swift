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

// MARK: - Native Liquid Glass Back Button Styler

final class LiquidGlassBackButtonView: UIView {
    let blurView: UIVisualEffectView
    let tintOverlayView: UIView
    let borderGradientLayer = CAGradientLayer()
    let borderMaskLayer = CAShapeLayer()
    let iconImageView: UIImageView

    init(size: CGFloat = 42.0) {
        let frame = CGRect(x: 0, y: 0, width: size, height: size)
        let radius = size / 2.0

        let blur = UIBlurEffect(style: .systemUltraThinMaterial)
        self.blurView = UIVisualEffectView(effect: blur)
        self.blurView.frame = frame
        self.blurView.layer.cornerRadius = radius
        self.blurView.layer.masksToBounds = true
        self.blurView.isUserInteractionEnabled = false

        self.tintOverlayView = UIView(frame: frame)
        self.tintOverlayView.layer.cornerRadius = radius
        self.tintOverlayView.layer.masksToBounds = true
        self.tintOverlayView.isUserInteractionEnabled = false

        self.borderGradientLayer.frame = frame
        self.borderGradientLayer.startPoint = CGPoint(x: 0.1, y: 0.0)
        self.borderGradientLayer.endPoint = CGPoint(x: 0.9, y: 1.0)
        self.borderMaskLayer.lineWidth = 1.0
        self.borderMaskLayer.path = UIBezierPath(ovalIn: frame.insetBy(dx: 0.5, dy: 0.5)).cgPath
        self.borderMaskLayer.fillColor = UIColor.clear.cgColor
        self.borderMaskLayer.strokeColor = UIColor.black.cgColor
        self.borderGradientLayer.mask = self.borderMaskLayer

        let config = UIImage.SymbolConfiguration(pointSize: 18.0, weight: .semibold)
        let img = UIImage(systemName: "chevron.backward", withConfiguration: config)
        self.iconImageView = UIImageView(image: img)
        self.iconImageView.tintColor = .label
        self.iconImageView.contentMode = .center
        self.iconImageView.frame = frame
        self.iconImageView.isUserInteractionEnabled = false

        super.init(frame: frame)
        self.isUserInteractionEnabled = false
        self.autoresizingMask = [.flexibleWidth, .flexibleHeight]

        self.layer.shadowColor = UIColor.black.cgColor
        self.layer.shadowOpacity = 0.14
        self.layer.shadowRadius = 8.0
        self.layer.shadowOffset = CGSize(width: 0.0, height: 2.0)
        self.layer.masksToBounds = false

        addSubview(blurView)
        blurView.contentView.addSubview(tintOverlayView)
        blurView.contentView.layer.addSublayer(borderGradientLayer)
        blurView.contentView.addSubview(iconImageView)

        updateAppearance()
    }

    required init?(coder: NSCoder) { fatalError("init(coder:) has not been implemented") }

    override func layoutSubviews() {
        super.layoutSubviews()
        let w = bounds.width > 0 ? bounds.width : 42.0
        let h = bounds.height > 0 ? bounds.height : 42.0
        let currentBounds = CGRect(x: 0, y: 0, width: w, height: h)
        let radius = w / 2.0

        blurView.frame = currentBounds
        blurView.layer.cornerRadius = radius
        tintOverlayView.frame = currentBounds
        tintOverlayView.layer.cornerRadius = radius
        borderGradientLayer.frame = currentBounds
        borderMaskLayer.path = UIBezierPath(ovalIn: currentBounds.insetBy(dx: 0.5, dy: 0.5)).cgPath
        iconImageView.frame = currentBounds
    }

    func updateAppearance() {
        let isDark = traitCollection.userInterfaceStyle == .dark
        tintOverlayView.backgroundColor = isDark ? UIColor(white: 1.0, alpha: 0.12) : UIColor(white: 1.0, alpha: 0.45)
        borderGradientLayer.colors = [
            (isDark ? UIColor(white: 1.0, alpha: 0.70) : UIColor(white: 1.0, alpha: 0.90)).cgColor,
            (isDark ? UIColor(white: 1.0, alpha: 0.15) : UIColor(white: 1.0, alpha: 0.30)).cgColor,
            (isDark ? UIColor(white: 1.0, alpha: 0.05) : UIColor(white: 1.0, alpha: 0.10)).cgColor,
            (isDark ? UIColor(white: 1.0, alpha: 0.40) : UIColor(white: 1.0, alpha: 0.55)).cgColor
        ]
    }

    override func traitCollectionDidChange(_ previousTraitCollection: UITraitCollection?) {
        super.traitCollectionDidChange(previousTraitCollection)
        updateAppearance()
    }
}

final class ButtonSpringAnimator: NSObject {
    static let shared = ButtonSpringAnimator()

    @objc func touchDown(_ sender: UIButton) {
        UIView.animate(withDuration: 0.12, delay: 0, options: [.beginFromCurrentState, .allowUserInteraction]) {
            sender.transform = CGAffineTransform(scaleX: 0.92, y: 0.92)
            sender.alpha = 0.82
        }
    }

    @objc func touchUp(_ sender: UIButton) {
        UIView.animate(withDuration: 0.35, delay: 0, usingSpringWithDamping: 0.6, initialSpringVelocity: 0.5, options: [.beginFromCurrentState, .allowUserInteraction]) {
            sender.transform = .identity
            sender.alpha = 1.0
        }
    }
}

@_cdecl("eschool_style_liquid_glass_button")
public func eschool_style_liquid_glass_button(buttonPtr: UnsafeMutableRawPointer) {
    let raw = Unmanaged<UIView>.fromOpaque(buttonPtr).takeUnretainedValue()
    NSLog("[LiquidGlass] Styling button: %@", raw)

    raw.backgroundColor = .clear
    raw.layer.backgroundColor = UIColor.clear.cgColor
    raw.layer.cornerRadius = 21.0
    raw.layer.masksToBounds = false
    raw.clipsToBounds = false

    if let btn = raw as? UIButton {
        btn.setTitle(nil, for: .normal)
        btn.setTitle(nil, for: .highlighted)
        btn.configuration = nil
        btn.titleLabel?.text = nil
        btn.titleLabel?.alpha = 0.0
        btn.backgroundColor = .clear
        btn.layer.backgroundColor = UIColor.clear.cgColor

        btn.addTarget(ButtonSpringAnimator.shared, action: #selector(ButtonSpringAnimator.touchDown(_:)), for: [.touchDown, .touchDragEnter])
        btn.addTarget(ButtonSpringAnimator.shared, action: #selector(ButtonSpringAnimator.touchUp(_:)), for: [.touchUpInside, .touchUpOutside, .touchCancel, .touchDragExit])
    }

    let tag = 99881
    if let existing = raw.viewWithTag(tag) {
        existing.removeFromSuperview()
    }

    let size: CGFloat = 42.0
    let glassView = LiquidGlassBackButtonView(size: size)
    glassView.tag = tag
    glassView.frame = CGRect(x: 0, y: 0, width: size, height: size)
    raw.insertSubview(glassView, at: 0)
}

// MARK: - Hardware-Synchronized Keyboard Button Gluer

final class KeyboardGluer: NSObject {
    static let shared = KeyboardGluer()
    private weak var targetView: UIView?
    private var currentLift: CGFloat = 0

    override init() {
        super.init()
        let center = NotificationCenter.default
        center.addObserver(
            self,
            selector: #selector(keyboardWillChange(_:)),
            name: UIResponder.keyboardWillChangeFrameNotification,
            object: nil
        )
        center.addObserver(
            self,
            selector: #selector(keyboardWillChange(_:)),
            name: UIResponder.keyboardWillShowNotification,
            object: nil
        )
        center.addObserver(
            self,
            selector: #selector(keyboardWillChange(_:)),
            name: UIResponder.keyboardWillHideNotification,
            object: nil
        )
    }

    func register(view: UIView) {
        NSLog("[KeyboardGluer] Registered view: %@", view)
        self.targetView = view
        if self.currentLift > 0 {
            view.transform = CGAffineTransform(translationX: 0, y: -self.currentLift)
        }
    }

    @objc private func keyboardWillChange(_ notification: Notification) {
        guard let view = self.targetView else {
            NSLog("[KeyboardGluer] targetView is nil on notification")
            return
        }

        guard let userInfo = notification.userInfo,
              let endFrameValue = userInfo[UIResponder.keyboardFrameEndUserInfoKey] as? NSValue
        else { return }

        let rawEndFrame = endFrameValue.cgRectValue
        let duration = (userInfo[UIResponder.keyboardAnimationDurationUserInfoKey] as? NSNumber)?.doubleValue ?? 0.25
        let curveValue = (userInfo[UIResponder.keyboardAnimationCurveUserInfoKey] as? NSNumber)?.intValue ?? 7

        guard let window = view.window ?? UIApplication.shared.connectedScenes
            .compactMap({ $0 as? UIWindowScene })
            .flatMap({ $0.windows })
            .first(where: { $0.isKeyWindow })
        else {
            NSLog("[KeyboardGluer] window is nil")
            return
        }

        let endFrame = window.convert(rawEndFrame, from: nil)
        let screenHeight = window.bounds.height
        let safeAreaBottom = window.safeAreaInsets.bottom

        let keyboardHeight: CGFloat = endFrame.origin.y < screenHeight ? (screenHeight - endFrame.origin.y) : 0
        let lift: CGFloat = keyboardHeight > 0 ? max(0, keyboardHeight - safeAreaBottom + 8.0) : 0
        self.currentLift = lift

        NSLog("[KeyboardGluer] kbHeight=%.1f safeAreaBottom=%.1f lift=%.1f dur=%.2f", keyboardHeight, safeAreaBottom, lift, duration)

        let options = UIView.AnimationOptions(rawValue: UInt(curveValue << 16))

        UIView.animate(
            withDuration: duration,
            delay: 0,
            options: [options, .beginFromCurrentState, .allowUserInteraction],
            animations: {
                view.transform = lift > 0 ? CGAffineTransform(translationX: 0, y: -lift) : .identity
            },
            completion: nil
        )
    }
}

@_cdecl("eschool_glue_button_to_keyboard")
public func eschool_glue_button_to_keyboard(viewPtr: UnsafeMutableRawPointer) {
    let raw = Unmanaged<UIView>.fromOpaque(viewPtr).takeUnretainedValue()
    KeyboardGluer.shared.register(view: raw)
}

// MARK: - Global Tap To Dismiss Keyboard

final class DismissKeyboardTapGesture: UITapGestureRecognizer {}

final class KeyboardDismisser: NSObject {
    static let shared = KeyboardDismisser()

    @objc func handleTap(_ gesture: UITapGestureRecognizer) {
        gesture.view?.endEditing(true)
        UIApplication.shared.sendAction(#selector(UIResponder.resignFirstResponder), to: nil, from: nil, for: nil)
    }
}

@_cdecl("eschool_install_tap_to_dismiss")
public func eschool_install_tap_to_dismiss() {
    DispatchQueue.main.async {
        let scenes = UIApplication.shared.connectedScenes.compactMap { $0 as? UIWindowScene }
        let windows = scenes.flatMap { $0.windows }
        let allWindows = windows.isEmpty ? UIApplication.shared.windows : windows

        for window in allWindows {
            if window.gestureRecognizers?.contains(where: { $0 is DismissKeyboardTapGesture }) == true {
                continue
            }
            let tap = DismissKeyboardTapGesture(target: KeyboardDismisser.shared, action: #selector(KeyboardDismisser.handleTap(_:)))
            tap.cancelsTouchesInView = false
            tap.requiresExclusiveTouchType = false
            window.addGestureRecognizer(tap)
        }
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

