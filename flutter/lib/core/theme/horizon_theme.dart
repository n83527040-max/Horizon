import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

/// Centralized design system. The palette follows the supplied reference:
/// dark navy, neon purple, electric blue, cyan, dark blue cards.
abstract final class HorizonPalette {
  static const Color background = Color(0xFF080B1D);
  static const Color primary = Color(0xFF8B35FF);
  static const Color secondary = Color(0xFF3565FF);
  static const Color accent = Color(0xFF20DDF5);
  static const Color card = Color(0xFF111B38);
  static const Color border = Color(0xFF2A3565);
  static const Color textPrimary = Color(0xFFFFFFFF);
  static const Color textSecondary = Color(0xFFAAB6D3);
  static const Color danger = Color(0xFFFF5C7A);
  static const Color success = Color(0xFF35E0A1);

  static const LinearGradient primaryGradient = LinearGradient(
    begin: Alignment.centerLeft,
    end: Alignment.centerRight,
    colors: <Color>[primary, secondary],
  );
  static const LinearGradient heroGradient = LinearGradient(
    begin: Alignment.topCenter,
    end: Alignment.bottomCenter,
    colors: <Color>[Color(0x00080B1D), Color(0xCC080B1D)],
  );
}

abstract final class HorizonRadii {
  static const double card = 20;
  static const double control = 16;
  static const double pill = 999;
}

abstract final class HorizonSpacing {
  static const double screen = 16;
  static const double card = 14;
  static const double gap = 12;
  static const double small = 8;
}

/// Responsive metrics computed from the real viewport, not fixed sizes.
class HorizonMetrics {
  const HorizonMetrics(this.size, this.padding);

  final Size size;
  final EdgeInsets padding;

  double get width => size.width;
  double get height => size.height;

  /// Phones use the full width; only wide tablets get a modest outer margin.
  double get horizontal {
    if (width >= 900) return 32;
    if (width >= 600) return 24;
    return HorizonSpacing.screen;
  }

  /// Two columns only when there is real room for them.
  int get gridColumns => width >= 720 ? 4 : (width >= 480 ? 2 : 2);
  int get catalogColumns => width >= 900 ? 2 : 1;
  bool get isTablet => width >= 600;
  double get heroHeight => (height * 0.23).clamp(150.0, 260.0);
  double get fontScale => (width / 393).clamp(0.88, 1.18);

  static HorizonMetrics of(BuildContext context) {
    final media = MediaQuery.of(context);
    return HorizonMetrics(media.size, media.padding);
  }
}

class HorizonTheme {
  const HorizonTheme._();

  static ThemeData dark() => _build(Brightness.dark);
  static ThemeData light() => _build(Brightness.light);

  static ThemeData _build(Brightness brightness) {
    final bool dark = brightness == Brightness.dark;
    final Color background = dark ? HorizonPalette.background : const Color(0xFFF3F5FF);
    final Color surface = dark ? HorizonPalette.card : const Color(0xFFFFFFFF);
    final Color onSurface = dark ? HorizonPalette.textPrimary : const Color(0xFF10152E);
    final Color muted = dark ? HorizonPalette.textSecondary : const Color(0xFF5A6484);

    final ColorScheme scheme = ColorScheme.fromSeed(
      seedColor: HorizonPalette.primary,
      brightness: brightness,
      primary: HorizonPalette.primary,
      secondary: HorizonPalette.secondary,
      tertiary: HorizonPalette.accent,
      surface: surface,
      error: HorizonPalette.danger,
    );

    final TextTheme baseText = ThemeData(brightness: brightness).textTheme;
    final TextTheme text = baseText.apply(
      bodyColor: onSurface,
      displayColor: onSurface,
      decorationColor: onSurface,
    );

    return ThemeData(
      useMaterial3: true,
      brightness: brightness,
      colorScheme: scheme,
      scaffoldBackgroundColor: background,
      canvasColor: background,
      cardColor: surface,
      dividerColor: dark ? HorizonPalette.border : const Color(0xFFDCE1F2),
      textTheme: text,
      primaryTextTheme: text,
      splashFactory: InkSplash.splashFactory,
      appBarTheme: AppBarTheme(
        backgroundColor: background,
        foregroundColor: onSurface,
        elevation: 0,
        centerTitle: false,
        systemOverlayStyle: dark ? SystemUiOverlayStyle.light : SystemUiOverlayStyle.dark,
      ),
      iconTheme: IconThemeData(color: onSurface, size: 22),
      inputDecorationTheme: InputDecorationTheme(
        filled: true,
        fillColor: surface,
        hintStyle: TextStyle(color: muted),
        contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 14),
        border: OutlineInputBorder(
          borderRadius: BorderRadius.circular(HorizonRadii.control),
          borderSide: BorderSide(color: dark ? HorizonPalette.border : const Color(0xFFDCE1F2)),
        ),
        enabledBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(HorizonRadii.control),
          borderSide: BorderSide(color: dark ? HorizonPalette.border : const Color(0xFFDCE1F2)),
        ),
        focusedBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(HorizonRadii.control),
          borderSide: const BorderSide(color: HorizonPalette.primary, width: 1.6),
        ),
      ),
      bottomSheetTheme: BottomSheetThemeData(
        backgroundColor: surface,
        modalBackgroundColor: surface,
        shape: const RoundedRectangleBorder(
          borderRadius: BorderRadius.vertical(top: Radius.circular(28)),
        ),
      ),
      snackBarTheme: SnackBarThemeData(
        backgroundColor: surface,
        contentTextStyle: TextStyle(color: onSurface),
        behavior: SnackBarBehavior.floating,
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(14)),
      ),
    );
  }
}
