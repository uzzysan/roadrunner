import { Ionicons } from '@expo/vector-icons';
import React, { type ComponentProps } from 'react';
import { StyleSheet, View, type ViewProps } from 'react-native';
import { useTheme } from '../hooks/useTheme';
import { AppText } from './AppText';

export type StatusBadgeTone = 'success' | 'warning' | 'danger' | 'info' | 'neutral' | 'offline' | 'stale';

export interface StatusBadgeProps extends ViewProps {
  label: string;
  tone?: StatusBadgeTone;
  icon?: ComponentProps<typeof Ionicons>['name'];
}

export function StatusBadge({ label, tone = 'neutral', icon, style, ...props }: StatusBadgeProps) {
  const { colors, theme } = useTheme();
  const palette = {
    success: { background: colors.successContainer, foreground: colors.onSuccessContainer, icon: 'checkmark-circle-outline' },
    warning: { background: colors.warningContainer, foreground: colors.onWarningContainer, icon: 'warning-outline' },
    danger: { background: colors.dangerContainer, foreground: colors.onDangerContainer, icon: 'alert-circle-outline' },
    info: { background: colors.infoContainer, foreground: colors.onInfoContainer, icon: 'information-circle-outline' },
    neutral: { background: colors.surfaceMuted, foreground: colors.textSecondary, icon: 'ellipse-outline' },
    offline: { background: colors.warningContainer, foreground: colors.onWarningContainer, icon: 'cloud-offline-outline' },
    stale: { background: colors.warningContainer, foreground: colors.onWarningContainer, icon: 'time-outline' },
  }[tone];

  return (
    <View
      accessibilityLabel={label}
      accessibilityRole="text"
      style={[
        styles.base,
        {
          backgroundColor: palette.background,
          borderRadius: theme.radius.pill,
          paddingHorizontal: theme.space.sm,
          paddingVertical: theme.space.xs,
        },
        style,
      ]}
      {...props}
    >
      <Ionicons
        accessibilityElementsHidden
        color={palette.foreground}
        importantForAccessibility="no-hide-descendants"
        name={icon ?? palette.icon as ComponentProps<typeof Ionicons>['name']}
        size={16}
      />
      <AppText style={{ color: palette.foreground }} variant="label">{label}</AppText>
    </View>
  );
}

const styles = StyleSheet.create({
  base: { alignItems: 'center', alignSelf: 'flex-start', flexDirection: 'row', gap: 4 },
});
