import { useFocusEffect } from '@react-navigation/native';
import axios from 'axios';
import React, { useCallback, useState } from 'react';
import { ActivityIndicator, RefreshControl, StyleSheet, View } from 'react-native';
import { useTranslation } from 'react-i18next';
import { apiClient } from '../api/client';
import { AppText } from '../components/AppText';
import { Card } from '../components/Card';
import { EmptyState } from '../components/EmptyState';
import { ErrorState } from '../components/ErrorState';
import { Screen } from '../components/Screen';
import { StatusBadge, type StatusBadgeTone } from '../components/StatusBadge';
import { useTheme } from '../hooks/useTheme';

interface TicketRecord {
  id: string;
  ticket_type: string;
  status: string;
  price: number;
  currency: string;
  created_at: string;
  valid_until: string;
  used_at?: string | null;
  route_id?: string | null;
  start_stop_id?: string | null;
  end_stop_id?: string | null;
}

type TicketsStatus = 'loading' | 'ready' | 'empty' | 'error' | 'offline' | 'stale';

interface TicketsState {
  data: TicketRecord[];
  status: TicketsStatus;
  refreshing: boolean;
  updatedAt?: Date;
}

const initialState: TicketsState = {
  data: [],
  status: 'loading',
  refreshing: false,
};

const statusTone = (status: string): StatusBadgeTone => ({
  pending: 'warning',
  ready: 'info',
  active: 'success',
  used: 'neutral',
  expired: 'warning',
  cancelled: 'danger',
  verification_error: 'danger',
})[status] as StatusBadgeTone ?? 'neutral';

export function TicketsScreen() {
  const { t, i18n } = useTranslation();
  const { colors, theme } = useTheme();
  const [state, setState] = useState<TicketsState>(initialState);

  const loadTickets = useCallback(async (refresh = false) => {
    setState((current) => ({
      ...current,
      refreshing: refresh || current.data.length > 0,
      status: current.data.length > 0 ? current.status : 'loading',
    }));
    try {
      const response = await apiClient.get<TicketRecord[]>('/tickets');
      if (!Array.isArray(response.data)) {
        throw new Error('Unexpected tickets response');
      }
      const data = response.data;
      setState({
        data,
        status: data.length > 0 ? 'ready' : 'empty',
        refreshing: false,
        updatedAt: new Date(),
      });
    } catch (error: unknown) {
      const offline = axios.isAxiosError(error) && !error.response;
      setState((current) => ({
        ...current,
        status: current.data.length > 0 ? (offline ? 'offline' : 'stale') : (offline ? 'offline' : 'error'),
        refreshing: false,
      }));
    }
  }, []);

  useFocusEffect(useCallback(() => {
    void loadTickets(false);
  }, [loadTickets]));

  const formatDate = (value: string) => {
    const date = new Date(value);
    return Number.isNaN(date.getTime())
      ? t('common.unknown')
      : new Intl.DateTimeFormat(i18n.language, {
        dateStyle: 'medium',
        timeStyle: 'short',
      }).format(date);
  };

  const formatPrice = (ticket: TicketRecord) => {
    try {
      return new Intl.NumberFormat(i18n.language, {
        style: 'currency',
        currency: ticket.currency,
      }).format(ticket.price);
    } catch {
      return `${ticket.price} ${ticket.currency}`;
    }
  };

  const ticketTypeLabel = (ticketType: string) => t(`tickets.type.${ticketType}`, {
    defaultValue: t('tickets.type.unknown'),
  });
  const ticketStatusLabel = (status: string) => t(`tickets.status.${status}`, {
    defaultValue: t('tickets.status.unknown'),
  });
  const showInitialError = state.data.length === 0 && (state.status === 'error' || state.status === 'offline');

  return (
    <Screen
      scroll
      contentContainerStyle={[styles.content, { gap: theme.space.xl }]}
      edges={['left', 'right', 'bottom']}
      scrollViewProps={{
        refreshControl: (
          <RefreshControl
            accessibilityLabel={t('tickets.refresh')}
            colors={[colors.primary]}
            onRefresh={() => void loadTickets(true)}
            refreshing={state.refreshing}
            tintColor={colors.primary}
          />
        ),
      }}
    >
      <View style={{ gap: theme.space.xs }}>
        <AppText accessibilityRole="header" variant="h1">{t('tickets.myTickets')}</AppText>
        <AppText tone="secondary">{t('tickets.description')}</AppText>
      </View>

      {state.status === 'loading' ? (
        <Card accessibilityLiveRegion="polite" style={[styles.center, { gap: theme.space.md }]}>
          <ActivityIndicator accessibilityLabel={t('tickets.loading')} color={colors.primary} />
          <AppText tone="secondary">{t('tickets.loading')}</AppText>
        </Card>
      ) : null}

      {showInitialError ? (
        <ErrorState
          description={state.status === 'offline'
            ? t('common.offlineDescription')
            : t('tickets.loadError')}
          onRetry={() => void loadTickets(false)}
          title={state.status === 'offline' ? t('common.offline') : t('common.error')}
        />
      ) : null}

      {state.status === 'empty' ? (
        <EmptyState
          description={t('tickets.noTicketsDescription')}
          icon="ticket-outline"
          title={t('tickets.noTickets')}
        />
      ) : null}

      {state.data.length > 0 ? (
        <View style={{ gap: theme.space.md }}>
          {state.status === 'offline' || state.status === 'stale' ? (
            <Card accessibilityLiveRegion="polite" style={{ gap: theme.space.sm }}>
              <StatusBadge
                label={state.status === 'offline' ? t('common.offline') : t('common.staleData')}
                tone={state.status === 'offline' ? 'offline' : 'stale'}
              />
              <AppText tone="secondary">
                {state.status === 'offline'
                  ? t('tickets.offlineWithData')
                  : t('tickets.staleWithData')}
              </AppText>
              {state.updatedAt ? (
                <AppText tone="muted" variant="caption">
                  {t('common.lastUpdated', { date: formatDate(state.updatedAt.toISOString()) })}
                </AppText>
              ) : null}
            </Card>
          ) : null}

          {state.data.map((ticket) => (
            <Card
              key={ticket.id}
              style={{ gap: theme.space.md }}
            >
              <View style={styles.row}>
                <View style={[styles.grow, { gap: theme.space.xs }]}>
                  <AppText accessibilityRole="header" variant="h3">
                    {ticketTypeLabel(ticket.ticket_type)}
                  </AppText>
                  <AppText tone="secondary" variant="caption">
                    {t('tickets.identifier', { id: ticket.id })}
                  </AppText>
                </View>
                <StatusBadge
                  label={ticketStatusLabel(ticket.status)}
                  tone={statusTone(ticket.status)}
                />
              </View>
              <View style={{ gap: theme.space.xs }}>
                <AppText variant="bodyStrong">{formatPrice(ticket)}</AppText>
                <AppText tone="secondary">
                  {t('tickets.purchasedAt')}: {formatDate(ticket.created_at)}
                </AppText>
                <AppText tone="secondary">
                  {t('tickets.validUntil')}: {formatDate(ticket.valid_until)}
                </AppText>
                {ticket.route_id ? (
                  <AppText tone="secondary">
                    {t('tickets.routeIdentifier', { id: ticket.route_id })}
                  </AppText>
                ) : null}
              </View>
            </Card>
          ))}
        </View>
      ) : null}

      <Card style={{ gap: theme.space.sm }}>
        <StatusBadge label={t('tickets.purchaseUnavailable')} tone="info" />
        <AppText tone="secondary">{t('tickets.purchaseUnavailableDescription')}</AppText>
      </Card>
    </Screen>
  );
}

const styles = StyleSheet.create({
  center: { alignItems: 'center' },
  content: { width: '100%' },
  grow: { flex: 1 },
  row: { alignItems: 'flex-start', flexDirection: 'row', flexWrap: 'wrap' },
});
