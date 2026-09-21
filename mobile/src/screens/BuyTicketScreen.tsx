import { useNavigation } from '@react-navigation/native';
import type { NativeStackNavigationProp } from '@react-navigation/native-stack';
import React from 'react';
import { StyleSheet, View } from 'react-native';
import { useTranslation } from 'react-i18next';
import { AppText } from '../components/AppText';
import { Button } from '../components/Button';
import { Card } from '../components/Card';
import { EmptyState } from '../components/EmptyState';
import { Screen } from '../components/Screen';
import { StatusBadge } from '../components/StatusBadge';
import { useTheme } from '../hooks/useTheme';
import type { RootStackParamList } from '../types';

type BuyTicketNavigationProp = NativeStackNavigationProp<RootStackParamList, 'BuyTicket'>;

export function BuyTicketScreen() {
  const { t } = useTranslation();
  const navigation = useNavigation<BuyTicketNavigationProp>();
  const { theme } = useTheme();

  return (
    <Screen
      scroll
      contentContainerStyle={[styles.content, { gap: theme.space.xl }]}
    >
      <View style={{ gap: theme.space.xs }}>
        <AppText accessibilityRole="header" variant="h1">{t('tickets.buyTicket')}</AppText>
        <AppText tone="secondary">{t('tickets.purchaseDescription')}</AppText>
      </View>

      <Card style={{ gap: theme.space.lg }}>
        <StatusBadge label={t('tickets.purchaseUnavailable')} tone="warning" />
        <EmptyState
          description={t('tickets.purchaseUnavailableDescription')}
          icon="card-outline"
          title={t('tickets.noOfferTitle')}
        />
        <AppText tone="secondary">
          {t('tickets.noOfferReason')}
        </AppText>
      </Card>

      {navigation.canGoBack() ? (
        <Button
          fullWidth
          icon="arrow-back-outline"
          label={t('common.back')}
          onPress={() => navigation.goBack()}
          variant="secondary"
        />
      ) : null}
    </Screen>
  );
}

const styles = StyleSheet.create({
  content: { width: '100%' },
});
