import 'package:fluent_ui/fluent_ui.dart';
import 'package:provider/provider.dart';

import '../../../providers/discovery.dart';
import '../../../utils/l10n.dart';

class DiscoveredDevicesList extends StatefulWidget {
  const DiscoveredDevicesList({super.key});

  @override
  State<DiscoveredDevicesList> createState() => _DiscoveredDevicesListState();
}

class _DiscoveredDevicesListState extends State<DiscoveredDevicesList> {
  late final DiscoveryProvider _provider;
  String selectedFingerprint = "";

  String? _selectedFingerprint;

  @override
  void initState() {
    super.initState();
    _provider = context.read<DiscoveryProvider>();
    _provider.startListening();
  }

  @override
  void dispose() {
    _provider.stopListening();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final s = S.of(context);

    return Consumer<DiscoveryProvider>(
      builder: (context, provider, _) {
        if (provider.error != null) {
          return Center(
            child: Text(s.error(provider.error!)),
          );
        }

        if (provider.devices.isEmpty) {
          return Center(
            child: Text(s.noDevicesFound),
          );
        }

        return ListView.builder(
          itemCount: provider.devices.length,
          itemBuilder: (context, index) {
            final device = provider.devices.values.elementAt(index);
            final isSelected = _selectedFingerprint == device.fingerprint;

            return ListTile.selectable(
              title: _DeviceTitle(
                alias: device.alias,
                isSelected: isSelected,
                device: device,
                onPairPressed: () => _handlePairDevice(device),
              ),
              subtitle: isSelected
                  ? Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        _buildDetailItem(s.model, device.deviceModel),
                        _buildDetailItem(s.type, device.deviceType),
                        _buildDetailItem(s.fingerprint, device.fingerprint),
                        _buildDetailItem(
                          s.lastSeen,
                          '${device.lastSeen.toLocal()}',
                        ),
                        _buildDetailItem(
                          s.ipAddresses,
                          device.ips.join(', '),
                        ),
                        FilledButton(
                          onPressed: () => _handlePairDevice(device),
                          child: Text(S.of(context).pair),
                        ),
                      ],
                    )
                  : Text(device.fingerprint),
              selected: isSelected,
              onSelectionChange: (v) =>
                  setState(() => selectedFingerprint = device.fingerprint),
            );
          },
        );
      },
    );
  }

  Widget _buildDetailItem(String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4.0),
      child: RichText(
        text: TextSpan(
          style: TextStyle(fontWeight: FontWeight.w600),
          children: [
            TextSpan(
              text: '$label: ',
              style: const TextStyle(fontWeight: FontWeight.bold),
            ),
            TextSpan(text: value),
          ],
        ),
      ),
    );
  }

  void _handlePairDevice(DiscoveredDevice device) {
    // PairDeviceRequest(
    //   fingerprint: device.fingerprint,
    //   alias: device.alias,
    // ).sendSignalToRust();
  }
}

class _DeviceTitle extends StatelessWidget {
  final String alias;
  final bool isSelected;
  final DiscoveredDevice device;
  final VoidCallback onPairPressed;

  const _DeviceTitle({
    required this.alias,
    required this.isSelected,
    required this.device,
    required this.onPairPressed,
  });

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        Expanded(
          child: Text(
            alias,
            style: TextStyle(fontWeight: FontWeight.w600),
          ),
        ),
        if (isSelected)
          Row(
            children: [
              FilledButton(
                onPressed: onPairPressed,
                child: Text(S.of(context).pair),
              ),
              const SizedBox(width: 12),
            ],
          ),
      ],
    );
  }
}
