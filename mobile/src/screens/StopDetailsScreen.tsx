import axios from 'axios';
import type { NativeStackScreenProps } from '@react-navigation/native-stack';
import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { Linking, Pressable, StyleSheet, View } from 'react-native';
import { useTranslation } from 'react-i18next';
import { WebView } from 'react-native-webview';

import { apiClient } from '../api/client';
import { AppText, Button, Card, EmptyState, ErrorState, Screen, StatusBadge } from '../components';
import { useTheme } from '../hooks/useTheme';
import type { RootStackParamList, RouteAtStop, ScheduleWithRoute, Stop } from '../types';
import { classifyTransitState, getAccessibleLineColor, MAP_HTML_TEMPLATE, mapNavigationKind, serializeWebViewMessage, type TransitDataState } from './MapScreen';

type Props = NativeStackScreenProps<RootStackParamList, 'StopDetails'>;
type Day = 'weekday' | 'saturday' | 'sunday' | 'holiday';

function responseArray<T>(value: unknown, property: string): T[] {
  if (Array.isArray(value)) return value as T[];
  if (value && typeof value === 'object') {
    const nested = (value as Record<string, unknown>)[property];
    if (Array.isArray(nested)) return nested as T[];
  }
  return [];
}

function responseStop(value: unknown): Stop | null {
  if (!value || typeof value !== 'object') return null;
  const record = value as Record<string, unknown>;
  const candidate = record.stop && typeof record.stop === 'object' ? record.stop as Record<string, unknown> : record;
  return typeof candidate.id === 'string' && typeof candidate.name === 'string' ? candidate as unknown as Stop : null;
}

export function directionsUrl(latitude: number, longitude: number): string | null {
  if (!Number.isFinite(latitude) || !Number.isFinite(longitude) || Math.abs(latitude) > 90 || Math.abs(longitude) > 180) return null;
  const lat = latitude.toFixed(6);
  const lon = longitude.toFixed(6);
  return `https://www.openstreetmap.org/?mlat=${lat}&mlon=${lon}#map=16/${lat}/${lon}`;
}

export function StopDetailsScreen({ navigation, route: navigationRoute }: Props) {
  const { t, i18n } = useTranslation();
  const { colors, theme, webViewTokens } = useTheme();
  const webViewRef = useRef<WebView>(null);
  const [stop, setStop] = useState<Stop | null>(null);
  const [routes, setRoutes] = useState<RouteAtStop[]>([]);
  const [schedules, setSchedules] = useState<ScheduleWithRoute[]>([]);
  const [status, setStatus] = useState<TransitDataState>('loading');
  const [updatedAt, setUpdatedAt] = useState<string | null>(null);
  const [selectedRoute, setSelectedRoute] = useState<string | null>(null);
  const [day, setDay] = useState<Day>('weekday');
  const [mapReady, setMapReady] = useState(false);
  const [mapUnavailable, setMapUnavailable] = useState(false);
  const stopId = navigationRoute.params.stopId;
  const linePalette = useCallback((candidate: unknown) => getAccessibleLineColor(candidate, colors.primaryContainer, colors.text, colors.onInverse, colors.onPrimaryContainer), [colors]);

  const load = useCallback(async (refresh = false) => {
    setStatus(refresh && stop ? 'refreshing' : 'loading');
    const path = `/stops/${encodeURIComponent(stopId)}`;
    const [stopResult, routesResult, schedulesResult] = await Promise.allSettled([apiClient.get(path), apiClient.get(`${path}/routes`), apiClient.get(`${path}/schedules`)]);
    const failed = [stopResult, routesResult, schedulesResult].filter(result => result.status === 'rejected');
    const offline = failed.some(result => result.status === 'rejected' && axios.isAxiosError(result.reason) && !result.reason.response);
    const nextStop = stopResult.status === 'fulfilled' ? responseStop(stopResult.value.data) : stop;
    if (stopResult.status === 'fulfilled') setStop(nextStop);
    if (routesResult.status === 'fulfilled') setRoutes(responseArray<RouteAtStop>(routesResult.value.data, 'routes'));
    if (schedulesResult.status === 'fulfilled') setSchedules(responseArray<ScheduleWithRoute>(schedulesResult.value.data, 'schedules'));
    setStatus(classifyTransitState({ hasData: Boolean(nextStop), offline, partial: failed.length > 0 && failed.length < 3, failed: failed.length === 3 }));
    if (failed.length === 0) setUpdatedAt(new Date().toISOString());
  }, [stop, stopId]);

  useEffect(() => { void load(); }, [stopId]);
  useEffect(() => {
    if (!stop || mapReady || mapUnavailable) return;
    const timeout = setTimeout(() => setMapUnavailable(true), 12000);
    return () => clearTimeout(timeout);
  }, [mapReady, mapUnavailable, stop]);
  useEffect(() => {
    if (!mapReady || !stop) return;
    webViewRef.current?.postMessage(serializeWebViewMessage({ type: 'initialize', tokens: webViewTokens, language: i18n.language, labels: {
      map: t('transit_stop.map_label', { name: stop.name }), stop: t('transit_map.stop_marker'), userLocation: t('transit_map.user_location'), routes: t('transit_map.routes_label'),
    } }));
    webViewRef.current?.postMessage(serializeWebViewMessage({ type: 'setStops', stops: [{ id: stop.id, name: stop.name, address: stop.address ?? '', latitude: stop.latitude, longitude: stop.longitude, routes: routes.map(item => ({ number: item.route_number, palette: linePalette(item.route_color) })) }] }));
    webViewRef.current?.postMessage(serializeWebViewMessage({ type: 'setView', latitude: stop.latitude, longitude: stop.longitude, zoom: 16 }));
  }, [i18n.language, linePalette, mapReady, routes, stop, t, webViewTokens]);

  const allowNavigation = useCallback((request: { url: string }) => { const kind = mapNavigationKind(request.url); if (kind === 'internal') return true; if (kind === 'attribution') void Linking.openURL(request.url); return false; }, []);
  const onMapMessage = useCallback((event: { nativeEvent: { data: string } }) => {
    try { const message = JSON.parse(event.nativeEvent.data) as Record<string, unknown>; if (message.type === 'mapReady') { setMapReady(true); setMapUnavailable(false); } else if (message.type === 'mapUnavailable' || message.type === 'bridgeError') setMapUnavailable(true); }
    catch (_error) { setMapUnavailable(true); }
  }, []);

  const departures = useMemo(() => {
    const groups = new Map<string, { name: string; number: string; color: string; times: Set<string> }>();
    schedules.forEach(schedule => {
      if (selectedRoute && schedule.route_id !== selectedRoute) return;
      if (schedule.day_type !== day && schedule.day_type !== 'everyday') return;
      const time = /^(\d{1,2}):(\d{2})/.exec(schedule.departure_time);
      if (!time) return;
      const group = groups.get(schedule.route_id) ?? { name: schedule.route_name, number: schedule.route_number, color: schedule.route_color, times: new Set<string>() };
      group.times.add(`${time[1].padStart(2, '0')}:${time[2]}`);
      groups.set(schedule.route_id, group);
    });
    return [...groups.entries()].map(([id, group]) => ({ id, ...group, times: [...group.times].sort() }));
  }, [day, schedules, selectedRoute]);

  const formattedUpdatedAt = updatedAt ? new Intl.DateTimeFormat(i18n.language === 'pl' ? 'pl-PL' : 'en-GB', { dateStyle: 'short', timeStyle: 'short' }).format(new Date(updatedAt)) : t('transit_common.not_available');
  if (status === 'loading') return <Screen contentContainerStyle={styles.centered}><StatusBadge label={t('transit_common.loading')} tone="info" /></Screen>;
  if ((status === 'error' || status === 'offline') && !stop) return <Screen contentContainerStyle={styles.centered}><ErrorState description={t(status === 'offline' ? 'transit_common.offline_description' : 'transit_stop.load_error_description')} onRetry={() => void load()} title={t(status === 'offline' ? 'transit_common.offline' : 'transit_stop.load_error_title')} /></Screen>;
  if (!stop) return <Screen contentContainerStyle={styles.centered}><EmptyState description={t('transit_stop.not_found_description')} icon="location-outline" title={t('transit_stop.not_found_title')} /></Screen>;

  const mapUrl = directionsUrl(stop.latitude, stop.longitude);
  return <Screen padded={false} scroll contentContainerStyle={{ paddingBottom: theme.space.xl }}>
    <View style={styles.mapContainer}>
      <WebView accessibilityLabel={t('transit_stop.map_label', { name: stop.name })} domStorageEnabled={false} javaScriptEnabled mixedContentMode="never" onError={() => setMapUnavailable(true)} onHttpError={() => setMapUnavailable(true)} onMessage={onMapMessage} onShouldStartLoadWithRequest={allowNavigation} originWhitelist={['about:blank', 'https://app.roadrunner.invalid']} ref={webViewRef} source={{ html: MAP_HTML_TEMPLATE, baseUrl: 'https://app.roadrunner.invalid/' }} style={styles.map} />
      {mapUnavailable ? <View style={[styles.mapFallback, { backgroundColor: colors.surface }]}><ErrorState description={t('transit_map.map_unavailable_description')} onRetry={() => { setMapReady(false); setMapUnavailable(false); webViewRef.current?.reload(); }} title={t('transit_map.map_unavailable_title')} /></View> : null}
    </View>
    <View style={{ gap: theme.space.lg, padding: theme.space.lg }}>
      <View style={{ gap: theme.space.xs }}><AppText accessibilityRole="header" variant="h1">{stop.name}</AppText>{stop.address ? <AppText tone="secondary">{stop.address}</AppText> : null}</View>
      {mapUrl ? <Button label={t('transit_stop.directions')} onPress={() => void Linking.openURL(mapUrl)} variant="secondary" /> : null}
      {stop.amenities?.length ? <Card><AppText variant="h3">{t('transit_stop.amenities')}</AppText><View style={styles.amenities}>{stop.amenities.map(amenity => <StatusBadge key={amenity} label={t(`amenities.${amenity}`, { defaultValue: amenity })} tone="info" />)}</View></Card> : null}
      <Card><View style={styles.meta}><AppText tone="secondary" variant="caption">{t('transit_common.source')}: {t('transit_route.source_value')}</AppText><AppText tone="secondary" variant="caption">{t('transit_common.retrieved_at')}: {formattedUpdatedAt}</AppText><AppText tone="secondary" variant="caption">{t('transit_common.schedule_kind')}: {t('transit_common.scheduled')}</AppText><AppText tone="secondary" variant="caption">{t('transit_common.timezone')}: {t('transit_common.timezone_unavailable')}</AppText>{status === 'stale' ? <StatusBadge label={t('transit_common.stale')} tone="stale" /> : status === 'partial' ? <StatusBadge label={t('transit_common.partial')} tone="warning" /> : status === 'refreshing' ? <StatusBadge label={t('transit_common.refreshing')} tone="info" /> : null}<Button label={t('transit_common.refresh')} loading={status === 'refreshing'} onPress={() => void load(true)} variant="quiet" /></View></Card>
      <View style={{ gap: theme.space.sm }}><AppText accessibilityRole="header" variant="h2">{t('transit_stop.serving_routes')}</AppText>{routes.length ? routes.map(item => { const palette = linePalette(item.route_color); const selected = selectedRoute === item.route_id; return <Card key={item.route_id} style={selected ? { backgroundColor: colors.primaryContainer, borderColor: colors.primary } : undefined}><Pressable accessibilityLabel={t('transit_stop.route_label', { number: item.route_number, name: item.route_name })} accessibilityRole="button" accessibilityState={{ selected }} onPress={() => setSelectedRoute(selected ? null : item.route_id)} style={[styles.routeRow, { minHeight: theme.size.touch }]}><View style={[styles.routeBadge, { backgroundColor: palette.background, borderRadius: theme.radius.md }]}><AppText style={{ color: palette.foreground }} variant="label">{item.route_number}</AppText></View><AppText style={styles.flex} variant="bodyStrong">{item.route_name}</AppText></Pressable><Button label={t('transit_stop.route_details')} onPress={() => navigation.navigate('RouteDetails', { routeId: item.route_id })} variant="quiet" /></Card>; }) : <EmptyState description={t('transit_stop.no_routes_description')} icon="bus-outline" title={t('transit_stop.no_routes_title')} />}</View>
      <View style={{ gap: theme.space.sm }}><AppText accessibilityRole="header" variant="h2">{t('transit_stop.schedule')}</AppText><View accessibilityRole="tablist" style={styles.dayTabs}>{(['weekday', 'saturday', 'sunday', 'holiday'] as const).map(item => <Pressable accessibilityRole="tab" accessibilityState={{ selected: item === day }} key={item} onPress={() => setDay(item)} style={[styles.dayTab, { borderColor: item === day ? colors.primary : colors.borderStrong, backgroundColor: item === day ? colors.primaryContainer : colors.surface, borderRadius: theme.radius.md, minHeight: theme.size.touch }]}><AppText style={{ color: item === day ? colors.onPrimaryContainer : colors.text }} variant="label">{t(`dayTypes.${item}`)}</AppText></Pressable>)}</View><AppText tone="secondary" variant="caption">{t('transit_common.scheduled')} · {t('transit_common.timezone_unavailable')}</AppText>{departures.length ? departures.map(group => { const palette = linePalette(group.color); return <Card key={group.id}><View style={styles.routeRow}><View style={[styles.routeBadge, { backgroundColor: palette.background, borderRadius: theme.radius.md }]}><AppText style={{ color: palette.foreground }} variant="label">{group.number}</AppText></View><AppText style={styles.flex} variant="bodyStrong">{group.name}</AppText></View><View style={styles.departures}>{group.times.map(time => <View key={time} style={{ backgroundColor: colors.surfaceMuted, borderRadius: theme.radius.sm, padding: theme.space.sm }}><AppText style={styles.tabular} variant="label">{time}</AppText></View>)}</View></Card>; }) : <EmptyState description={t('transit_stop.no_schedule_description')} icon="calendar-outline" title={t('transit_stop.no_schedule_title')} />}</View>
    </View>
  </Screen>;
}

export default StopDetailsScreen;

const styles = StyleSheet.create({
  amenities: { flexDirection: 'row', flexWrap: 'wrap', gap: 8, marginTop: 8 }, centered: { alignItems: 'center', flex: 1, justifyContent: 'center' }, dayTab: { alignItems: 'center', borderWidth: 1, flexGrow: 1, justifyContent: 'center', minWidth: '48%', paddingHorizontal: 4 }, dayTabs: { flexDirection: 'row', flexWrap: 'wrap', gap: 4 }, departures: { flexDirection: 'row', flexWrap: 'wrap', gap: 8, marginTop: 12 }, flex: { flex: 1 }, map: { flex: 1 }, mapContainer: { height: 220, position: 'relative' }, mapFallback: { ...StyleSheet.absoluteFillObject, alignItems: 'center', justifyContent: 'center', padding: 16 }, meta: { alignItems: 'flex-start', gap: 8 }, routeBadge: { alignItems: 'center', justifyContent: 'center', minHeight: 44, minWidth: 44, padding: 8 }, routeRow: { alignItems: 'center', flexDirection: 'row', gap: 12 }, tabular: { fontVariant: ['tabular-nums'] },
});
