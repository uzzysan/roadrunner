import AsyncStorage from '@react-native-async-storage/async-storage';
import React, { createContext, useCallback, useEffect, useMemo, useState } from 'react';
import {
  AccessibilityInfo,
  ActivityIndicator,
  StyleSheet,
  View,
  useColorScheme,
} from 'react-native';
import { PaperProvider } from 'react-native-paper';
import { createAppTheme, serializeThemeTokens } from './createTheme';
import type {
  AppColors,
  AppTheme,
  SerializableThemeTokens,
  ThemePreference,
} from './types';

const THEME_PREFERENCE_STORAGE_KEY = '@roadrunner_theme_preference_v2';

export interface ThemeContextValue {
  /** Complete app theme. */
  theme: AppTheme;
  colors: AppColors;
  preference: ThemePreference;
  setPreference: (preference: ThemePreference) => Promise<void>;
  isDark: boolean;
  reducedMotion: boolean;
  webViewTokens: SerializableThemeTokens;
}

export const ThemeContext = createContext<ThemeContextValue | null>(null);

const isThemePreference = (value: string | null): value is ThemePreference =>
  value === 'system' || value === 'light' || value === 'dark';

export interface RoadRunnerThemeProviderProps {
  children: React.ReactNode;
}

export function RoadRunnerThemeProvider({ children }: RoadRunnerThemeProviderProps) {
  const systemScheme = useColorScheme();
  const [preference, setPreferenceState] = useState<ThemePreference>('system');
  const [isHydrated, setIsHydrated] = useState(false);
  const [reducedMotion, setReducedMotion] = useState(false);

  useEffect(() => {
    let active = true;
    AsyncStorage.getItem(THEME_PREFERENCE_STORAGE_KEY)
      .then((storedPreference) => {
        if (active && isThemePreference(storedPreference)) {
          setPreferenceState(storedPreference);
        }
      })
      .catch(() => undefined)
      .finally(() => {
        if (active) setIsHydrated(true);
      });
    return () => {
      active = false;
    };
  }, []);

  useEffect(() => {
    let active = true;
    AccessibilityInfo.isReduceMotionEnabled().then((enabled) => {
      if (active) setReducedMotion(enabled);
    });
    const subscription = AccessibilityInfo.addEventListener(
      'reduceMotionChanged',
      setReducedMotion,
    );
    return () => {
      active = false;
      subscription.remove();
    };
  }, []);

  const setPreference = useCallback(async (nextPreference: ThemePreference) => {
    setPreferenceState(nextPreference);
    try {
      await AsyncStorage.setItem(THEME_PREFERENCE_STORAGE_KEY, nextPreference);
    } catch {
      // The in-memory preference remains useful for this session.
    }
  }, []);

  const resolvedScheme = preference === 'system'
    ? (systemScheme === 'dark' ? 'dark' : 'light')
    : preference;
  const theme = useMemo(() => createAppTheme(resolvedScheme), [resolvedScheme]);
  const webViewTokens = useMemo(
    () => serializeThemeTokens(theme, reducedMotion),
    [reducedMotion, theme],
  );
  const value = useMemo<ThemeContextValue>(() => ({
    theme,
    colors: theme.colors,
    preference,
    setPreference,
    isDark: theme.dark,
    reducedMotion,
    webViewTokens,
  }), [preference, reducedMotion, setPreference, theme, webViewTokens]);

  if (!isHydrated) {
    return (
      <View
        accessibilityLabel="RoadRunner"
        accessibilityRole="progressbar"
        style={[styles.bootstrap, { backgroundColor: theme.colors.background }]}
      >
        <ActivityIndicator color={theme.colors.primary} />
      </View>
    );
  }

  return (
    <ThemeContext.Provider value={value}>
      <PaperProvider theme={theme.paperTheme}>{children}</PaperProvider>
    </ThemeContext.Provider>
  );
}

const styles = StyleSheet.create({
  bootstrap: {
    alignItems: 'center',
    flex: 1,
    justifyContent: 'center',
  },
});

export { THEME_PREFERENCE_STORAGE_KEY };
