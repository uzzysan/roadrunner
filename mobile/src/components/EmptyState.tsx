import { Ionicons } from '@expo/vector-icons';
import React, { type ComponentProps } from 'react';
import { StyleSheet, View, type ViewProps } from 'react-native';
import { useTheme } from '../hooks/useTheme';
import { AppText } from './AppText';
import { Button } from './Button';

export interface EmptyStateProps extends ViewProps {
  title: string;
  description?: string;
  icon?: ComponentProps<typeof Ionicons>['name'];
  actionLabel?: string;
  onAction?: () => void;
}

export function EmptyState({
  actionLabel,
  description,
  icon = 'file-tray-outline',
  onAction,
  style,
  title,
  ...props
}: EmptyStateProps) {
  const { colors, theme } = useTheme();
  return (
    <View
      accessibilityRole="summary"
      style={[styles.base, { gap: theme.space.md, padding: theme.space.xl }, style]}
      {...props}
    >
      <Ionicons accessibilityElementsHidden color={colors.textSecondary} name={icon} size={48} />
      <AppText style={styles.center} variant="h3">{title}</AppText>
      {description ? <AppText style={styles.center} tone="secondary">{description}</AppText> : null}
      {actionLabel && onAction ? <Button label={actionLabel} onPress={onAction} variant="secondary" /> : null}
    </View>
  );
}

const styles = StyleSheet.create({
  base: { alignItems: 'center', justifyContent: 'center' },
  center: { textAlign: 'center' },
});
