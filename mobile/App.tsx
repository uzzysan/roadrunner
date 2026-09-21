import React, { useEffect, useState } from 'react';
import { StatusBar } from 'expo-status-bar';
import { ActivityIndicator, StyleSheet, View, useColorScheme } from 'react-native';
import { SafeAreaProvider } from 'react-native-safe-area-context';
import { initializeI18n } from './src/i18n';
import { useTheme } from './src/hooks/useTheme';
import { AppNavigator } from './src/navigation/AppNavigator';
import { RoadRunnerThemeProvider, getSemanticColors } from './src/theme';

function Bootstrap() {
  const systemScheme = useColorScheme() === 'dark' ? 'dark' : 'light';
  const colors = getSemanticColors(systemScheme);
  return (
    <View accessibilityLabel="RoadRunner" accessibilityRole="progressbar" style={[styles.bootstrap, { backgroundColor: colors.background }]}>
      <ActivityIndicator color={colors.primary} />
    </View>
  );
}

function ThemedApplication() {
  const { isDark } = useTheme();
  return (
    <>
      <StatusBar style={isDark ? 'light' : 'dark'} />
      <AppNavigator />
    </>
  );
}

export default function App() {
  const [i18nReady, setI18nReady] = useState(false);

  useEffect(() => {
    let active = true;
    initializeI18n().then(() => {
      if (active) setI18nReady(true);
    }).catch((error) => {
      if (__DEV__) console.error('i18n initialization failed', error);
    });
    return () => {
      active = false;
    };
  }, []);

  if (!i18nReady) return <Bootstrap />;

  return (
    <SafeAreaProvider>
      <RoadRunnerThemeProvider>
        <ThemedApplication />
      </RoadRunnerThemeProvider>
    </SafeAreaProvider>
  );
}

const styles = StyleSheet.create({
  bootstrap: { alignItems: 'center', flex: 1, justifyContent: 'center' },
});
