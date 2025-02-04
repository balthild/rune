import 'package:fluent_ui/fluent_ui.dart';

class FingerprintFigure extends StatelessWidget {
  const FingerprintFigure({
    super.key,
    required this.fingerprint,
  });

  final String? fingerprint;

  @override
  Widget build(BuildContext context) {
    final theme = FluentTheme.of(context);

    final localFingerprint = fingerprint;

    return LayoutBuilder(
      builder: (context, constraints) {
        return GridView.count(
          crossAxisCount: 4,
          childAspectRatio: 2,
          mainAxisSpacing: 4,
          crossAxisSpacing: 4,
          physics: const NeverScrollableScrollPhysics(),
          shrinkWrap: true,
          children: List.generate(20, (index) {
            final startIndex = index * 2;
            final text = localFingerprint == null ||
                    startIndex >= localFingerprint.length
                ? ''
                : '${localFingerprint[startIndex]}${startIndex + 1 < localFingerprint.length ? localFingerprint[startIndex + 1] : ''}';
            return TextBox(
              readOnly: true,
              placeholder: text,
              placeholderStyle: TextStyle(
                color: theme.resources.textFillColorPrimary,
              ),
              style: const TextStyle(
                fontFamily: 'NotoRunic',
                fontSize: 20,
                letterSpacing: 4,
              ),
              textAlign: TextAlign.center,
            );
          }),
        );
      },
    );
  }
}
