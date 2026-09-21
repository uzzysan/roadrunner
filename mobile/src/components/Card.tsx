import React from 'react';
import {
  Pressable,
  type PressableProps,
  StyleSheet,
  type StyleProp,
  View,
  type ViewProps,
  type ViewStyle,
} from 'react-native';
import { useTheme } from '../hooks/useTheme';

export interface CardProps extends Omit<ViewProps, 'style'> {
  onPress?: PressableProps['onPress'];
  accessibilityLabel?: string;
  elevated?: boolean;
  padded?: boolean;
  style?: StyleProp<ViewStyle>;
}

export function Card({
  accessibilityLabel,
  children,
  elevated = false,
  onPress,
  padded = true,
  style,
  ...props
}: CardProps) {
  const { colors, theme } = useTheme();
  const cardStyle: StyleProp<ViewStyle> = [
    styles.base,
    {
      backgroundColor: elevated ? colors.surfaceRaised : colors.surface,
      borderColor: colors.border,
      borderRadius: theme.radius.lg,
      elevation: elevated ? 1 : 0,
      padding: padded ? theme.space.lg : 0,
    },
    style,
  ];

  if (onPress) {
    return (
      <Pressable
        accessibilityLabel={accessibilityLabel}
        accessibilityRole="button"
        onPress={onPress}
        style={({ pressed }) => [cardStyle, pressed && { backgroundColor: colors.surfaceMuted }]}
      >
        {children}
      </Pressable>
    );
  }
  return <View style={cardStyle} {...props}>{children}</View>;
}

const styles = StyleSheet.create({
  base: { borderWidth: StyleSheet.hairlineWidth },
});
