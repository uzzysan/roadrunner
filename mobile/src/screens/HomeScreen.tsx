import type { BottomTabNavigationProp } from '@react-navigation/bottom-tabs';
import { useNavigation } from '@react-navigation/native';
import React from 'react';
import { StyleSheet, View } from 'react-native';
import { useTranslation } from 'react-i18next';
import { AppText } from '../components/AppText';
import { BrandMark } from '../components/BrandMark';
import { Button } from '../components/Button';
import { Card } from '../components/Card';
import { Screen } from '../components/Screen';
import { useTheme } from '../hooks/useTheme';
import { useAuthStore } from '../store/authStore';
import type { MainTabParamList } from '../types';

type HomeNavigationProp = BottomTabNavigationProp<MainTabParamList, 'Home'>;

export function HomeScreen() {
  const { t } = useTranslation();
  const navigation = useNavigation<HomeNavigationProp>();
  const { user } = useAuthStore();
  const { theme } = useTheme();
  const displayName = user?.firstName?.trim() || t('home.traveler');

  return (
    <Screen
      scroll
      contentContainerStyle={[styles.content, { gap: theme.space.xl }]}
      edges={['left', 'right', 'bottom']}
    >
      <View style={{ gap: theme.space.lg }}>
        <BrandMark accessibilityLabel={t('brand.logoLabel')} />
        <View style={{ gap: theme.space.xs }}>
          <AppText accessibilityRole="header" variant="h1">
            {t('home.greeting', { name: displayName })}
          </AppText>
          <AppText tone="secondary">{t('home.description')}</AppText>
        </View>
      </View>

      <Card style={{ gap: theme.space.lg }}>
        <View style={{ gap: theme.space.xs }}>
          <AppText variant="h2">{t('home.planJourney')}</AppText>
          <AppText tone="secondary">{t('home.planJourneyDescription')}</AppText>
        </View>
        <Button
          fullWidth
          icon="map-outline"
          label={t('home.openMap')}
          onPress={() => navigation.navigate('Map')}
        />
      </Card>

      <Card style={{ gap: theme.space.lg }}>
        <View style={{ gap: theme.space.xs }}>
          <AppText variant="h2">{t('tickets.myTickets')}</AppText>
          <AppText tone="secondary">{t('home.ticketsDescription')}</AppText>
        </View>
        <Button
          fullWidth
          icon="ticket-outline"
          label={t('home.openTickets')}
          onPress={() => navigation.navigate('Tickets')}
          variant="secondary"
        />
      </Card>
    </Screen>
  );
}

const styles = StyleSheet.create({
  content: { width: '100%' },
});
