import React from 'react';
import {
  ScrollView,
  type ScrollViewProps,
  StyleSheet,
  type StyleProp,
  View,
  type ViewProps,
  type ViewStyle,
} from 'react-native';
import { SafeAreaView, type Edge } from 'react-native-safe-area-context';
import { useTheme } from '../hooks/useTheme';

export interface ScreenProps extends ViewProps {
  scroll?: boolean;
  edges?: readonly Edge[];
  contentContainerStyle?: StyleProp<ViewStyle>;
  scrollViewProps?: Omit<ScrollViewProps, 'children' | 'contentContainerStyle'>;
  padded?: boolean;
}

export function Screen({
  children,
  contentContainerStyle,
  edges = ['top', 'right', 'bottom', 'left'],
  padded = true,
  scroll = false,
  scrollViewProps,
  style,
  ...viewProps
}: ScreenProps) {
  const { colors, theme } = useTheme();
  const contentStyle = [
    styles.content,
    padded && { paddingHorizontal: theme.space.lg, paddingVertical: theme.space.lg },
    contentContainerStyle,
  ];

  return (
    <SafeAreaView
      edges={[...edges]}
      style={[styles.safeArea, { backgroundColor: colors.background }, style]}
      {...viewProps}
    >
      {scroll ? (
        <ScrollView
          keyboardShouldPersistTaps="handled"
          {...scrollViewProps}
          contentContainerStyle={contentStyle}
        >
          {children}
        </ScrollView>
      ) : (
        <View style={contentStyle}>{children}</View>
      )}
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  content: { flexGrow: 1 },
  safeArea: { flex: 1 },
});
