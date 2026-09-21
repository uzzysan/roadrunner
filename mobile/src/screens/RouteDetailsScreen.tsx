import axios from 'axios';
import { Ionicons } from '@expo/vector-icons';
import type { NativeStackScreenProps } from '@react-navigation/native-stack';
import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { Linking, Pressable, StyleSheet, View } from 'react-native';
import { useTranslation } from 'react-i18next';
import { WebView } from 'react-native-webview';

import { AppText, Button, Card, EmptyState, ErrorState, Screen, StatusBadge } from '../components';
import { apiClient } from '../api/client';
import { useTheme } from '../hooks/useTheme';
import type { RootStackParamList, Route, RouteSchedule, StopInRoute } from '../types';
import { classifyTransitState, getAccessibleLineColor, mapNavigationKind, serializeWebViewMessage, type TransitDataState } from './MapScreen';

type Props = NativeStackScreenProps<RootStackParamList, 'RouteDetails'>;
type RoutePayload = { route: Route; stops: StopInRoute[] };

export const ROUTE_MAP_HTML = `<!DOCTYPE html><html lang="en"><head>
<meta charset="utf-8"/><meta name="viewport" content="width=device-width,initial-scale=1"/>
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; script-src 'unsafe-inline' https://unpkg.com; style-src 'unsafe-inline' https://unpkg.com; img-src data: https://*.tile.openstreetmap.org; connect-src https://*.tile.openstreetmap.org"/>
<script>window.__rrLeafletFailed=false;window.__rrCssFailed=false;</script>
<link rel="stylesheet" href="https://unpkg.com/leaflet@1.9.4/dist/leaflet.css" onerror="window.__rrCssFailed=true"/>
<script src="https://unpkg.com/leaflet@1.9.4/dist/leaflet.js" onerror="window.__rrLeafletFailed=true"></script>
<style>:root{color-scheme:light dark;--background:Canvas;--surface:Canvas;--text:CanvasText;--primary:Highlight;--route:Highlight}html,body,#map{height:100%;width:100%;margin:0;background:var(--background)}.marker{box-sizing:border-box;width:26px;height:26px;border-radius:50%;border:4px solid var(--surface);outline:2px solid var(--text);background:var(--route)}.marker.endpoint{outline-width:4px}.popup{color:var(--text);font:600 16px/24px system-ui,-apple-system,sans-serif}.leaflet-popup-content-wrapper,.leaflet-popup-tip,.leaflet-control-zoom a{background:var(--surface);color:var(--text)}.leaflet-control-attribution{background:var(--surface)!important;color:var(--text)!important}.leaflet-control-attribution a{color:var(--primary)!important}</style>
</head><body><div id="map" role="img"></div><script>(function(){'use strict';const send=value=>window.ReactNativeWebView&&window.ReactNativeWebView.postMessage(JSON.stringify(value));if(window.__rrLeafletFailed||window.__rrCssFailed||typeof L==='undefined'){send({type:'mapUnavailable'});return}const map=L.map('map',{zoomControl:true,attributionControl:true}).setView([52.2297,21.0122],12);L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png',{maxZoom:19,attribution:'&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'}).on('tileerror',()=>send({type:'mapUnavailable'})).addTo(map);let markers=[];let line=null;const validColor=value=>typeof value==='string'&&/^#[0-9a-f]{6}$/i.test(value);const finite=value=>typeof value==='number'&&Number.isFinite(value);function applyTheme(tokens){if(!tokens||!tokens.colors)return;const root=document.documentElement.style;const values={'--background':tokens.colors.background,'--surface':tokens.colors.surface,'--text':tokens.colors.text,'--primary':tokens.colors.primary};Object.entries(values).forEach(([key,value])=>{if(validColor(value))root.setProperty(key,value)})}function clear(){markers.forEach(marker=>marker.remove());markers=[];if(line){line.remove();line=null}}function setRoute(message){clear();if(!message.palette||!validColor(message.palette.background)||!Array.isArray(message.stops))return;document.documentElement.style.setProperty('--route',message.palette.background);const points=[];message.stops.forEach((stop,index)=>{if(!stop||!finite(stop.latitude)||!finite(stop.longitude)||typeof stop.name!=='string')return;const endpoint=index===0||index===message.stops.length-1;const marker=L.marker([stop.latitude,stop.longitude],{icon:L.divIcon({className:'marker'+(endpoint?' endpoint':''),html:'',iconSize:[26,26],iconAnchor:[13,13]}),title:stop.name.slice(0,500),alt:stop.name.slice(0,500)});const popup=document.createElement('div');popup.className='popup';popup.textContent=stop.name.slice(0,500);marker.bindPopup(popup);marker.addTo(map);markers.push(marker);points.push([stop.latitude,stop.longitude])});if(points.length){line=L.polyline(points,{color:message.palette.background,weight:5,opacity:1}).addTo(map);map.fitBounds(line.getBounds(),{padding:[32,32]})}}function receive(event){try{const message=JSON.parse(event.data);if(!message||typeof message.type!=='string')return;if(message.type==='initialize'){applyTheme(message.tokens);document.documentElement.lang=typeof message.language==='string'?message.language:'en';document.getElementById('map').setAttribute('aria-label',typeof message.label==='string'?message.label:'')}else if(message.type==='setRoute')setRoute(message)}catch(_error){send({type:'bridgeError'})}}document.addEventListener('message',receive);window.addEventListener('message',receive);send({type:'mapReady'})}());</script></body></html>`;

function routePayload(value: unknown): RoutePayload | null {
  if (!value || typeof value !== 'object') return null;
  const record = value as Record<string, unknown>;
  if (!record.route || typeof record.route !== 'object') return null;
  return { route: record.route as Route, stops: Array.isArray(record.stops) ? record.stops as StopInRoute[] : [] };
}

function schedulesPayload(value: unknown): RouteSchedule[] {
  if (Array.isArray(value)) return value as RouteSchedule[];
  if (value && typeof value === 'object' && Array.isArray((value as Record<string, unknown>).schedules_by_stop)) return (value as { schedules_by_stop: RouteSchedule[] }).schedules_by_stop;
  return [];
}

function transportTime(value: string): string {
  const match = /^(\d{1,2}):(\d{2})/.exec(value);
  return match ? `${match[1].padStart(2, '0')}:${match[2]}` : value;
}

export default function RouteDetailsScreen({ navigation, route: navigationRoute }: Props) {
  const { t, i18n } = useTranslation();
  const { colors, theme, webViewTokens } = useTheme();
  const webViewRef = useRef<WebView>(null);
  const [route, setRoute] = useState<Route | null>(null);
  const [stops, setStops] = useState<StopInRoute[]>([]);
  const [schedules, setSchedules] = useState<RouteSchedule[]>([]);
  const [status, setStatus] = useState<TransitDataState>('loading');
  const [activeTab, setActiveTab] = useState<'stops' | 'schedule'>('stops');
  const [dayType, setDayType] = useState<'weekday' | 'saturday' | 'sunday'>('weekday');
  const [selectedStop, setSelectedStop] = useState<string | null>(null);
  const [updatedAt, setUpdatedAt] = useState<string | null>(null);
  const [mapReady, setMapReady] = useState(false);
  const [mapUnavailable, setMapUnavailable] = useState(false);
  const routeId = navigationRoute.params.routeId;

  const palette = useMemo(() => getAccessibleLineColor(route?.color, colors.primaryContainer, colors.text, colors.onInverse, colors.onPrimaryContainer), [colors, route?.color]);
  const load = useCallback(async (refresh = false) => {
    const hasData = Boolean(route);
    setStatus(refresh && hasData ? 'refreshing' : 'loading');
    const [routeResult, scheduleResult] = await Promise.allSettled([apiClient.get(`/routes/${routeId}`), apiClient.get(`/routes/${routeId}/schedules`)]);
    const failed = [routeResult, scheduleResult].filter(result => result.status === 'rejected');
    const offline = failed.some(result => result.status === 'rejected' && axios.isAxiosError(result.reason) && !result.reason.response);
    let nextRoute = route;
    if (routeResult.status === 'fulfilled') {
      const payload = routePayload(routeResult.value.data);
      nextRoute = payload?.route ?? null;
      setRoute(nextRoute); setStops(payload?.stops ?? []);
    }
    if (scheduleResult.status === 'fulfilled') setSchedules(schedulesPayload(scheduleResult.value.data));
    setStatus(classifyTransitState({ hasData: Boolean(nextRoute), offline, partial: failed.length === 1, failed: failed.length === 2 }));
    if (failed.length === 0) setUpdatedAt(new Date().toISOString());
  }, [route, routeId]);

  useEffect(() => { void load(); }, [routeId]);
  useEffect(() => {
    if (!route || mapReady || mapUnavailable) return;
    const timeout = setTimeout(() => setMapUnavailable(true), 12000);
    return () => clearTimeout(timeout);
  }, [mapReady, mapUnavailable, route]);
  useEffect(() => {
    if (!mapReady || !route) return;
    webViewRef.current?.postMessage(serializeWebViewMessage({ type: 'initialize', tokens: webViewTokens, language: i18n.language, label: t('transit_route.map_label', { route: route.number }) }));
    webViewRef.current?.postMessage(serializeWebViewMessage({ type: 'setRoute', palette, stops: stops.map(stop => ({ name: stop.name, latitude: stop.latitude, longitude: stop.longitude })) }));
  }, [i18n.language, mapReady, palette, route, stops, t, webViewTokens]);

  const handleMapMessage = useCallback((event: { nativeEvent: { data: string } }) => {
    try { const message = JSON.parse(event.nativeEvent.data) as Record<string, unknown>; if (message.type === 'mapReady') { setMapReady(true); setMapUnavailable(false); } else if (message.type === 'mapUnavailable' || message.type === 'bridgeError') setMapUnavailable(true); }
    catch (_error) { setMapUnavailable(true); }
  }, []);
  const allowNavigation = useCallback((request: { url: string }) => { const kind = mapNavigationKind(request.url); if (kind === 'internal') return true; if (kind === 'attribution') void Linking.openURL(request.url); return false; }, []);
  const formattedUpdatedAt = updatedAt ? new Intl.DateTimeFormat(i18n.language === 'pl' ? 'pl-PL' : 'en-GB', { dateStyle: 'short', timeStyle: 'short' }).format(new Date(updatedAt)) : t('transit_common.not_available');

  if (status === 'loading') return <Screen contentContainerStyle={styles.centered}><StatusBadge label={t('transit_common.loading')} tone="info" /></Screen>;
  if ((status === 'error' || status === 'offline') && !route) return <Screen contentContainerStyle={styles.centered}><ErrorState description={t(status === 'offline' ? 'transit_common.offline_description' : 'transit_route.load_error_description')} onRetry={() => void load()} title={t(status === 'offline' ? 'transit_common.offline' : 'transit_route.load_error_title')} /></Screen>;
  if (!route) return <Screen contentContainerStyle={styles.centered}><EmptyState description={t('transit_route.not_found_description')} icon="bus-outline" title={t('transit_route.not_found_title')} /></Screen>;

  const stateBadge = status === 'stale' ? <StatusBadge label={t('transit_common.stale')} tone="stale" /> : status === 'partial' ? <StatusBadge label={t('transit_common.partial')} tone="warning" /> : status === 'refreshing' ? <StatusBadge label={t('transit_common.refreshing')} tone="info" /> : null;
  const selectedSchedules = selectedStop ? schedules.filter(schedule => schedule.stop_id === selectedStop) : schedules;

  return <Screen padded={false} scroll contentContainerStyle={styles.grow}>
    <View style={styles.mapContainer}>
      <WebView accessibilityLabel={t('transit_route.map_label', { route: route.number })} domStorageEnabled={false} javaScriptEnabled mixedContentMode="never" onError={() => setMapUnavailable(true)} onHttpError={() => setMapUnavailable(true)} onMessage={handleMapMessage} onShouldStartLoadWithRequest={allowNavigation} originWhitelist={['about:blank', 'https://app.roadrunner.invalid']} ref={webViewRef} source={{ html: ROUTE_MAP_HTML, baseUrl: 'https://app.roadrunner.invalid/' }} style={styles.map} />
      {mapUnavailable ? <View style={[styles.mapFallback, { backgroundColor: colors.surface }]}><ErrorState description={t('transit_route.map_unavailable_description')} onRetry={() => { setMapReady(false); setMapUnavailable(false); webViewRef.current?.reload(); }} title={t('transit_route.map_unavailable_title')} /></View> : null}
    </View>
    <View style={[styles.body, { gap: theme.space.lg, padding: theme.space.lg }]}>
      <View style={styles.titleRow}><View style={[styles.routeBadge, { backgroundColor: palette.background, borderRadius: theme.radius.md }]}><AppText style={{ color: palette.foreground }} variant="h3">{route.number}</AppText></View><View style={styles.flex}><AppText accessibilityRole="header" variant="h1">{route.name}</AppText>{route.description ? <AppText tone="secondary">{route.description}</AppText> : null}</View></View>
      <Card><View style={styles.meta}><AppText tone="secondary" variant="caption">{t('transit_common.source')}: {t('transit_route.source_value')}</AppText><AppText tone="secondary" variant="caption">{t('transit_common.retrieved_at')}: {formattedUpdatedAt}</AppText><AppText tone="secondary" variant="caption">{t('transit_common.timezone')}: {t('transit_common.timezone_unavailable')}</AppText><AppText tone="secondary" variant="caption">{t('transit_common.schedule_kind')}: {t('transit_common.scheduled')}</AppText>{stateBadge}<Button label={t('transit_common.refresh')} loading={status === 'refreshing'} onPress={() => void load(true)} variant="quiet" /></View></Card>
      <View accessibilityRole="tablist" style={[styles.tabs, { borderColor: colors.borderStrong, borderRadius: theme.radius.md }]}>{(['stops', 'schedule'] as const).map(tab => <Pressable accessibilityRole="tab" accessibilityState={{ selected: activeTab === tab }} key={tab} onPress={() => setActiveTab(tab)} style={[styles.tab, { minHeight: theme.size.touch }, activeTab === tab && { backgroundColor: colors.primaryContainer }]}><Ionicons accessibilityElementsHidden color={activeTab === tab ? colors.onPrimaryContainer : colors.textSecondary} name={tab === 'stops' ? 'location-outline' : 'time-outline'} size={20} /><AppText style={{ color: activeTab === tab ? colors.onPrimaryContainer : colors.textSecondary }} variant="label">{t(`transit_route.${tab}`)}{tab === 'stops' ? ` (${stops.length})` : ''}</AppText></Pressable>)}</View>
      {activeTab === 'stops' ? (stops.length ? <View style={{ gap: theme.space.sm }}>{stops.map((stop, index) => <Card accessibilityLabel={t('transit_route.open_stop', { name: stop.name })} key={stop.id} onPress={() => { setSelectedStop(stop.id); navigation.navigate('StopDetails', { stopId: stop.id }); }} style={selectedStop === stop.id ? { backgroundColor: colors.primaryContainer, borderColor: colors.primary } : undefined}><View style={styles.stopRow}><View style={[styles.stopNumber, { backgroundColor: palette.background, borderRadius: theme.radius.pill }]}><AppText style={{ color: palette.foreground }} variant="label">{index + 1}</AppText></View><View style={styles.flex}><AppText variant="bodyStrong">{stop.name}</AppText>{stop.is_optional ? <StatusBadge label={t('transit_route.on_request')} tone="info" /> : null}</View><Ionicons accessibilityElementsHidden color={colors.textSecondary} name="chevron-forward-outline" size={theme.size.icon} /></View></Card>)}</View> : <EmptyState description={t('transit_route.no_stops_description')} icon="location-outline" title={t('transit_route.no_stops_title')} />) : <View style={{ gap: theme.space.lg }}>
        <View accessibilityRole="tablist" style={styles.dayTabs}>{(['weekday', 'saturday', 'sunday'] as const).map(day => <Pressable accessibilityRole="tab" accessibilityState={{ selected: dayType === day }} key={day} onPress={() => setDayType(day)} style={[styles.dayTab, { borderColor: dayType === day ? colors.primary : colors.borderStrong, minHeight: theme.size.touch }, dayType === day && { backgroundColor: colors.primaryContainer }]}><AppText style={{ color: dayType === day ? colors.onPrimaryContainer : colors.text }} variant="label">{t(`dayTypes.${day}`)}</AppText></Pressable>)}</View>
        {selectedSchedules.length ? selectedSchedules.map(schedule => { const departures = dayType === 'weekday' ? schedule.weekday_departures : dayType === 'saturday' ? schedule.saturday_departures : schedule.sunday_departures; return <Card key={schedule.stop_id}><AppText variant="h3">{schedule.stop_order}. {schedule.stop_name}</AppText><AppText tone="secondary" variant="caption">{t('transit_common.scheduled')} · {t('transit_common.timezone_unavailable')}</AppText>{departures.length ? <View style={styles.departures}>{departures.map((time, index) => <View key={`${time}-${index}`} style={[styles.departure, { backgroundColor: colors.surfaceMuted, borderRadius: theme.radius.sm }]}><AppText style={styles.tabular} variant="label">{transportTime(time)}</AppText></View>)}</View> : <AppText tone="secondary">{t('transit_route.no_departures_for_day')}</AppText>}</Card>; }) : <EmptyState description={t('transit_route.no_schedule_description')} icon="calendar-outline" title={t('transit_route.no_schedule_title')} />}
      </View>}
    </View>
  </Screen>;
}

const styles = StyleSheet.create({
  body: {}, centered: { alignItems: 'center', flex: 1, justifyContent: 'center' }, dayTab: { alignItems: 'center', borderBottomWidth: 2, flex: 1, justifyContent: 'center', paddingHorizontal: 4 }, dayTabs: { flexDirection: 'row' }, departure: { minWidth: 64, paddingHorizontal: 12, paddingVertical: 8 }, departures: { flexDirection: 'row', flexWrap: 'wrap', gap: 8, marginTop: 12 }, flex: { flex: 1 }, grow: { flexGrow: 1 }, map: { flex: 1 }, mapContainer: { height: 220, position: 'relative' }, mapFallback: { ...StyleSheet.absoluteFillObject, alignItems: 'center', justifyContent: 'center', padding: 16 }, meta: { alignItems: 'flex-start', gap: 8 }, routeBadge: { alignItems: 'center', justifyContent: 'center', minHeight: 56, minWidth: 56, padding: 8 }, stopNumber: { alignItems: 'center', height: 32, justifyContent: 'center', width: 32 }, stopRow: { alignItems: 'center', flexDirection: 'row', gap: 12 }, tab: { alignItems: 'center', flex: 1, flexDirection: 'row', gap: 8, justifyContent: 'center' }, tabs: { borderWidth: 1, flexDirection: 'row', overflow: 'hidden' }, tabular: { fontVariant: ['tabular-nums'] }, titleRow: { alignItems: 'center', flexDirection: 'row', gap: 16 },
});
