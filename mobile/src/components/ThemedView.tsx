import React from 'react';
import { View, type ViewProps } from 'react-native';
import { useTheme } from '../hooks/useTheme';

/** Compatibility wrapper for existing screens; prefer Screen for screen roots. */
export function ThemedView({ style, ...props }: ViewProps) {
  const { colors } = useTheme();
  return <View style={[{ backgroundColor: colors.background }, style]} {...props} />;
}
