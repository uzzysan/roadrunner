import React from 'react';
import { Text, type TextProps } from 'react-native';
import { useTheme } from '../hooks/useTheme';

export type AppTextVariant =
  | 'display'
  | 'h1'
  | 'h2'
  | 'h3'
  | 'body'
  | 'bodyStrong'
  | 'label'
  | 'caption'
  | 'metric';
export type AppTextTone = 'default' | 'secondary' | 'muted' | 'primary' | 'danger' | 'success' | 'warning' | 'info' | 'inverse';

export interface AppTextProps extends TextProps {
  variant?: AppTextVariant;
  tone?: AppTextTone;
}

export function AppText({
  allowFontScaling = true,
  children,
  style,
  tone = 'default',
  variant = 'body',
  ...props
}: AppTextProps) {
  const { colors, theme } = useTheme();
  const typography = theme.typography[variant];
  const toneColor = {
    default: colors.text,
    secondary: colors.textSecondary,
    muted: colors.textMuted,
    primary: colors.primary,
    danger: colors.danger,
    success: colors.success,
    warning: colors.warning,
    info: colors.info,
    inverse: colors.onInverse,
  }[tone];

  return (
    <Text
      allowFontScaling={allowFontScaling}
      style={[
        {
          color: toneColor,
          fontSize: typography.size,
          fontWeight: String(typography.weight) as '400' | '600' | '700',
          lineHeight: typography.lineHeight,
        },
        style,
      ]}
      {...props}
    >
      {children}
    </Text>
  );
}
