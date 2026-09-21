import { Ionicons } from '@expo/vector-icons';
import React, { type ComponentProps, type ReactNode } from 'react';
import {
  ActivityIndicator,
  Pressable,
  type PressableProps,
  StyleSheet,
  type StyleProp,
  ViewStyle,
} from 'react-native';
import { useTheme } from '../hooks/useTheme';
import { AppText } from './AppText';

export type ButtonVariant = 'primary' | 'secondary' | 'quiet' | 'danger';

export interface ButtonProps extends Omit<PressableProps, 'children' | 'style'> {
  children?: ReactNode;
  label?: string;
  title?: string;
  variant?: ButtonVariant;
  icon?: ComponentProps<typeof Ionicons>['name'];
  loading?: boolean;
  loadingLabel?: string;
  safety?: boolean;
  fullWidth?: boolean;
  style?: StyleProp<ViewStyle>;
}

export function Button({
  accessibilityLabel,
  children,
  disabled,
  fullWidth = false,
  icon,
  label,
  loading = false,
  loadingLabel,
  safety = false,
  style,
  title,
  variant = 'primary',
  ...props
}: ButtonProps) {
  const { colors, theme } = useTheme();
  const text = label ?? title ?? children;
  const defaultAccessibilityLabel = typeof text === 'string' ? text : undefined;
  const isDisabled = disabled || loading;
  const palette = {
    primary: { background: colors.primary, foreground: colors.onPrimary, border: colors.primary },
    secondary: { background: colors.surface, foreground: colors.primary, border: colors.borderStrong },
    quiet: { background: 'transparent', foreground: colors.primary, border: 'transparent' },
    danger: { background: colors.danger, foreground: colors.onDanger, border: colors.danger },
  }[variant];

  return (
    <Pressable
      accessibilityLabel={loading
        ? loadingLabel ?? accessibilityLabel ?? defaultAccessibilityLabel
        : accessibilityLabel ?? defaultAccessibilityLabel}
      accessibilityRole="button"
      accessibilityState={{ busy: loading, disabled: isDisabled }}
      disabled={isDisabled}
      hitSlop={safety ? 4 : undefined}
      style={({ pressed }) => [
        styles.base,
        {
          backgroundColor: isDisabled ? colors.disabled : pressed ? colors.primaryPressed : palette.background,
          borderColor: isDisabled ? colors.disabled : palette.border,
          minHeight: safety ? theme.size.touchSafety : theme.size.touch,
          minWidth: safety ? theme.size.touchSafety : theme.size.touch,
          opacity: pressed && !isDisabled ? 0.94 : 1,
        },
        fullWidth && styles.fullWidth,
        style,
      ]}
      {...props}
    >
      {loading ? (
        <ActivityIndicator color={isDisabled ? colors.onDisabled : palette.foreground} />
      ) : (
        <>
          {icon ? <Ionicons color={isDisabled ? colors.onDisabled : palette.foreground} name={icon} size={20} /> : null}
          <AppText
            accessibilityElementsHidden
            importantForAccessibility="no"
            style={{ color: isDisabled ? colors.onDisabled : palette.foreground }}
            variant="bodyStrong"
          >
            {text}
          </AppText>
        </>
      )}
    </Pressable>
  );
}

const styles = StyleSheet.create({
  base: {
    alignItems: 'center',
    borderRadius: 12,
    borderWidth: 1,
    flexDirection: 'row',
    gap: 8,
    justifyContent: 'center',
    paddingHorizontal: 16,
  },
  fullWidth: { alignSelf: 'stretch' },
});
