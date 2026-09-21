import React, { useState } from 'react';
import { Alert, StyleSheet, View } from 'react-native';
import { useTranslation } from 'react-i18next';
import { authApi } from '../api/client';
import { AppText } from '../components/AppText';
import { Button } from '../components/Button';
import { Card } from '../components/Card';
import { ErrorState } from '../components/ErrorState';
import { Screen } from '../components/Screen';
import { StatusBadge } from '../components/StatusBadge';
import { changeLanguage, type LanguageCode } from '../i18n';
import { useTheme } from '../hooks/useTheme';
import { useAuthStore } from '../store/authStore';
import type { ThemePreference } from '../theme/types';

const themeOptions: ThemePreference[] = ['system', 'light', 'dark'];
const languageOptions: LanguageCode[] = ['pl', 'en'];

export function ProfileScreen() {
  const { t, i18n } = useTranslation();
  const { user, logout, refreshToken } = useAuthStore();
  const { colors, preference, setPreference, theme } = useTheme();
  const [isLoggingOut, setIsLoggingOut] = useState(false);
  const [isChangingLanguage, setIsChangingLanguage] = useState(false);
  const [languageError, setLanguageError] = useState(false);

  const handleLogout = async () => {
    setIsLoggingOut(true);
    try {
      if (refreshToken) await authApi.logout(refreshToken);
    } catch {
      // Ending the local session remains safe when server revocation is temporarily unavailable.
    } finally {
      logout();
      setIsLoggingOut(false);
    }
  };

  const confirmLogout = () => {
    Alert.alert(
      t('profile.logoutTitle'),
      t('profile.logoutDescription'),
      [
        { text: t('common.cancel'), style: 'cancel' },
        { text: t('auth.logout'), onPress: handleLogout, style: 'destructive' },
      ],
    );
  };

  const selectLanguage = async (language: LanguageCode) => {
    setLanguageError(false);
    setIsChangingLanguage(true);
    try {
      await changeLanguage(language);
    } catch {
      setLanguageError(true);
    } finally {
      setIsChangingLanguage(false);
    }
  };

  const themeLabel = (option: ThemePreference) => ({
    system: t('settings.systemMode'),
    light: t('settings.lightMode'),
    dark: t('settings.darkMode'),
  })[option];
  const languageLabel = (option: LanguageCode) => ({
    pl: t('settings.polish'),
    en: t('settings.english'),
  })[option];
  const initials = [user?.firstName?.[0], user?.lastName?.[0]].filter(Boolean).join('');

  return (
    <Screen
      scroll
      contentContainerStyle={[styles.content, { gap: theme.space.xl }]}
      edges={['left', 'right', 'bottom']}
    >
      <Card elevated style={[styles.identity, { gap: theme.space.sm }]}>
        <View
          accessibilityElementsHidden
          importantForAccessibility="no-hide-descendants"
          style={[
            styles.avatar,
            {
              backgroundColor: colors.primaryContainer,
              borderRadius: theme.radius.pill,
              height: theme.size.touchSafety,
              width: theme.size.touchSafety,
            },
          ]}
        >
          <AppText style={{ color: colors.onPrimaryContainer }} variant="h2">
            {initials || t('profile.defaultInitial')}
          </AppText>
        </View>
        <AppText accessibilityRole="header" style={styles.center} variant="h1">
          {[user?.firstName, user?.lastName].filter(Boolean).join(' ') || t('profile.unknownUser')}
        </AppText>
        {user?.email ? <AppText tone="secondary">{user.email}</AppText> : null}
        {user?.role ? (
          <StatusBadge
            icon="person-circle-outline"
            label={t(`profile.role.${user.role}`, { defaultValue: user.role })}
            tone="info"
          />
        ) : null}
      </Card>

      <Card style={{ gap: theme.space.lg }}>
        <View style={{ gap: theme.space.xs }}>
          <AppText variant="h2">{t('profile.theme')}</AppText>
          <AppText tone="secondary">{t('profile.themeDescription')}</AppText>
        </View>
        <View accessibilityRole="radiogroup" style={{ gap: theme.space.sm }}>
          {themeOptions.map((option) => (
            <Button
              accessibilityRole="radio"
              accessibilityState={{ selected: preference === option }}
              fullWidth
              key={option}
              label={themeLabel(option)}
              onPress={() => void setPreference(option)}
              variant={preference === option ? 'primary' : 'secondary'}
            />
          ))}
        </View>
      </Card>

      <Card style={{ gap: theme.space.lg }}>
        <View style={{ gap: theme.space.xs }}>
          <AppText variant="h2">{t('profile.language')}</AppText>
          <AppText tone="secondary">{t('profile.languageDescription')}</AppText>
        </View>
        <View accessibilityRole="radiogroup" style={{ gap: theme.space.sm }}>
          {languageOptions.map((option) => {
            const selected = i18n.language.startsWith(option);
            return (
              <Button
                accessibilityRole="radio"
                accessibilityState={{ selected }}
                disabled={isChangingLanguage}
                fullWidth
                key={option}
                label={languageLabel(option)}
                onPress={() => void selectLanguage(option)}
                variant={selected ? 'primary' : 'secondary'}
              />
            );
          })}
        </View>
        {languageError ? (
          <ErrorState
            description={t('profile.languageChangeError')}
            title={t('common.error')}
          />
        ) : null}
      </Card>

      <Button
        fullWidth
        label={t('auth.logout')}
        loading={isLoggingOut}
        loadingLabel={t('profile.loggingOut')}
        onPress={confirmLogout}
        variant="danger"
      />
    </Screen>
  );
}

const styles = StyleSheet.create({
  avatar: { alignItems: 'center', justifyContent: 'center' },
  center: { textAlign: 'center' },
  content: { width: '100%' },
  identity: { alignItems: 'center' },
});
