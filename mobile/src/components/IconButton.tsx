import { Ionicons } from '@expo/vector-icons';
import React, { type ComponentProps } from 'react';
import { Pressable, type PressableProps, StyleSheet, type StyleProp, type ViewStyle } from 'react-native';
import { useTheme } from '../hooks/useTheme';

export interface IconButtonProps extends Omit<PressableProps, 'children' | 'style'> {
  icon: ComponentProps<typeof Ionicons>['name'];
  accessibilityLabel: string;
  safety?: boolean;
  selected?: boolean;
  color?: string;
  style?: StyleProp<ViewStyle>;
}

export function IconButton({
  accessibilityLabel,
  color,
  disabled,
  icon,
  safety = false,
  selected = false,
  style,
  ...props
}: IconButtonProps) {
  const { colors, theme } = useTheme();
  const target = safety ? theme.size.touchSafety : theme.size.touch;
  return (
    <Pressable
      accessibilityLabel={accessibilityLabel}
      accessibilityRole="button"
      accessibilityState={{ disabled: Boolean(disabled), selected }}
      disabled={disabled}
      style={({ pressed }) => [
        styles.base,
        {
          backgroundColor: selected ? colors.primaryContainer : pressed ? colors.surfaceMuted : 'transparent',
          height: target,
          opacity: disabled ? 0.65 : 1,
          width: target,
        },
        style,
      ]}
      {...props}
    >
      <Ionicons
        accessibilityElementsHidden
        color={disabled ? colors.onDisabled : color ?? colors.primary}
        importantForAccessibility="no-hide-descendants"
        name={icon}
        size={theme.size.icon}
      />
    </Pressable>
  );
}

const styles = StyleSheet.create({
  base: {
    alignItems: 'center',
    borderRadius: 12,
    justifyContent: 'center',
  },
});
