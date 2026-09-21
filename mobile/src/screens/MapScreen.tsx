import axios from 'axios';
import { Ionicons } from '@expo/vector-icons';
import type { CompositeScreenProps } from '@react-navigation/native';
import type { BottomTabScreenProps } from '@react-navigation/bottom-tabs';
import type { NativeStackScreenProps } from '@react-navigation/native-stack';
import * as Location from 'expo-location';
import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { FlatList, Linking, Pressable, StyleSheet, View } from 'react-native';
import { useTranslation } from 'react-i18next';
import { WebView } from 'react-native-webview';

import { AppText, Button, Card, EmptyState, ErrorState, IconButton, Screen, StatusBadge, TextField } from '../components';
import { apiClient } from '../api/client';
import { useTheme } from '../hooks/useTheme';
import type { MainTabParamList, RootStackParamList, Route, Stop } from '../types';

export type TransitDataState = 'loading' | 'refreshing' | 'success' | 'empty' | 'error' | 'offline' | 'partial' | 'stale';
type LinePalette = { background: string; foreground: string };
type MapScreenProps = CompositeScreenProps<BottomTabScreenProps<MainTabParamList, 'Map'>, NativeStackScreenProps<RootStackParamList>>;
type ThemeContextValue = ReturnType<typeof useTheme>;
type MapBridgeMessage =
  | { type: 'initialize'; tokens: ThemeContextValue['webViewTokens']; language: string; labels: Record<'map' | 'stop' | 'userLocation' | 'routes', string> }
  | { type: 'setStops'; stops: Array<{ id: string; name: string; address: string; latitude: number; longitude: number; routes: Array<{ number: string; palette: LinePalette }> }> }
  | { type: 'setUserLocation'; latitude: number; longitude: number }
  | { type: 'setView'; latitude: number; longitude: number; zoom: number }
  | { type: 'centerOnUser' };

const LINE_COLOR_PATTERN = /^#[0-9a-f]{6}$/i;
const TRUSTED_EXTERNAL_URLS = ['https://www.openstreetmap.org/', 'https://openstreetmap.org/'];

export function mapNavigationKind(rawUrl: string): 'internal' | 'attribution' | 'blocked' {
  if (rawUrl === 'about:blank') return 'internal';
  try {
    const url = new URL(rawUrl);
    if (url.protocol === 'https:' && url.hostname === 'app.roadrunner.invalid') return 'internal';
    if (url.protocol === 'https:' && TRUSTED_EXTERNAL_URLS.some(trusted => url.origin === new URL(trusted).origin) && url.pathname === '/copyright') return 'attribution';
  } catch (_error) { /* Invalid URLs never leave the WebView. */ }
  return 'blocked';
}

function rgbFromHex(value: string): [number, number, number] | null {
  if (!LINE_COLOR_PATTERN.test(value)) return null;
  const integer = Number.parseInt(value.slice(1), 16);
  return [(integer >> 16) & 255, (integer >> 8) & 255, integer & 255];
}

function relativeLuminance(value: string): number | null {
  const rgb = rgbFromHex(value);
  if (!rgb) return null;
  const channels = rgb.map(channel => {
    const normalized = channel / 255;
    return normalized <= 0.04045 ? normalized / 12.92 : ((normalized + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2];
}

function contrastRatio(first: string, second: string): number {
  const firstLuminance = relativeLuminance(first);
  const secondLuminance = relativeLuminance(second);
  if (firstLuminance === null || secondLuminance === null) return 0;
  return (Math.max(firstLuminance, secondLuminance) + 0.05) / (Math.min(firstLuminance, secondLuminance) + 0.05);
}

export function getAccessibleLineColor(candidate: unknown, fallbackBackground: string, darkForeground: string, lightForeground: string, fallbackForeground: string): LinePalette {
  if (typeof candidate !== 'string' || !LINE_COLOR_PATTERN.test(candidate)) return { background: fallbackBackground, foreground: fallbackForeground };
  const background = candidate.toUpperCase();
  const darkRatio = contrastRatio(background, darkForeground);
  const lightRatio = contrastRatio(background, lightForeground);
  const foreground = darkRatio >= lightRatio ? darkForeground : lightForeground;
  return Math.max(darkRatio, lightRatio) >= 4.5 ? { background, foreground } : { background: fallbackBackground, foreground: fallbackForeground };
}

export function classifyTransitState(input: { hasData: boolean; loading?: boolean; refreshing?: boolean; offline?: boolean; partial?: boolean; failed?: boolean }): TransitDataState {
  if (input.loading) return 'loading';
  if (input.refreshing) return 'refreshing';
  if (input.offline) return input.hasData ? 'stale' : 'offline';
  if (input.partial) return input.hasData ? 'partial' : 'error';
  if (input.failed) return input.hasData ? 'stale' : 'error';
  return input.hasData ? 'success' : 'empty';
}

export function getSelectedStopPalette(colors: { primary: string; onPrimary: string }): { fill: string; ring: string; foreground: string } {
  return { fill: colors.primary, ring: colors.primary, foreground: colors.onPrimary };
}

export function serializeWebViewMessage(message: unknown): string {
  const serialized = JSON.stringify(message, (key, value: unknown) => {
    if (key === '__proto__' || key === 'constructor' || key === 'prototype') return undefined;
    if (typeof value === 'number' && !Number.isFinite(value)) return null;
    return value;
  }) ?? '{}';
  return serialized.replace(/\u2028/g, '\\u2028').replace(/\u2029/g, '\\u2029');
}

function asArray<T>(value: unknown, property?: string): T[] {
  if (Array.isArray(value)) return value as T[];
  if (property && value && typeof value === 'object') {
    const nested = (value as Record<string, unknown>)[property];
    if (Array.isArray(nested)) return nested as T[];
  }
  return [];
}

export const MAP_HTML_TEMPLATE = `<!DOCTYPE html>
<html lang="en"><head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width,initial-scale=1" />
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; script-src 'unsafe-inline' https://unpkg.com; style-src 'unsafe-inline' https://unpkg.com; img-src data: https://*.tile.openstreetmap.org; connect-src https://*.tile.openstreetmap.org" />
<script>window.__rrLeafletFailed=false;window.__rrCssFailed=false;</script>
<link rel="stylesheet" href="https://unpkg.com/leaflet@1.9.4/dist/leaflet.css" onerror="window.__rrCssFailed=true" />
<script src="https://unpkg.com/leaflet@1.9.4/dist/leaflet.js" onerror="window.__rrLeafletFailed=true"></script>
<style>
:root{color-scheme:light dark;--background:Canvas;--surface:Canvas;--text:CanvasText;--secondary:GrayText;--primary:Highlight;--on-primary:HighlightText;--selected:Highlight;--radius:12px}
html,body,#map{height:100%;width:100%;margin:0;padding:0;background:var(--background);color:var(--text)}body{font-family:system-ui,-apple-system,sans-serif}
.stop-marker,.user-marker{box-sizing:border-box;border:3px solid var(--surface);outline:2px solid var(--text);border-radius:50%;background:var(--primary)}
.stop-marker{width:24px;height:24px}.stop-marker.selected{width:30px;height:30px;outline:4px solid var(--selected)}
.user-marker{width:20px;height:20px;background:var(--on-primary);outline-color:var(--primary)}
.popup{min-width:160px;color:var(--text)}.popup-title{margin:0 0 4px;font-size:16px;font-weight:600}.popup-meta{margin:0;color:var(--secondary);font-size:14px}
.leaflet-popup-content-wrapper,.leaflet-popup-tip,.leaflet-control-zoom a{background:var(--surface);color:var(--text)}.leaflet-control-attribution{background:var(--surface)!important;color:var(--text)!important}.leaflet-control-attribution a{color:var(--primary)!important}
.route-list{display:flex;flex-wrap:wrap;gap:4px;margin-top:8px}.route-badge{border-radius:var(--radius);padding:4px 8px;font-size:14px;font-weight:600}
</style></head><body><div id="map" role="img"></div><script>
(function(){'use strict';
const send=payload=>window.ReactNativeWebView&&window.ReactNativeWebView.postMessage(JSON.stringify(payload));
if(window.__rrLeafletFailed||window.__rrCssFailed||typeof L==='undefined'){send({type:'mapUnavailable',reason:'dependency'});return}
let labels={map:'',stop:'',userLocation:'',routes:''};const markers=new Map();let selectedStopId=null;let userMarker=null;
const map=L.map('map',{zoomControl:true,attributionControl:true}).setView([52.2297,21.0122],12);
L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png',{maxZoom:19,attribution:'&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'}).on('tileerror',()=>send({type:'mapUnavailable',reason:'tiles'})).addTo(map);
const validColor=value=>typeof value==='string'&&/^#[0-9a-f]{6}$/i.test(value);const finite=value=>typeof value==='number'&&Number.isFinite(value);const text=value=>typeof value==='string'?value.slice(0,500):'';
function applyTheme(tokens){if(!tokens||!tokens.colors)return;const root=document.documentElement.style;const pairs={'--background':tokens.colors.background,'--surface':tokens.colors.surface,'--text':tokens.colors.text,'--secondary':tokens.colors.textSecondary,'--primary':tokens.colors.primary,'--on-primary':tokens.colors.onPrimary,'--selected':tokens.colors.primary};Object.entries(pairs).forEach(([name,value])=>{if(validColor(value))root.setProperty(name,value)});if(tokens.radius&&Number.isFinite(tokens.radius.md))root.setProperty('--radius',tokens.radius.md+'px')}
function popupFor(stop){const root=document.createElement('div');root.className='popup';const title=document.createElement('p');title.className='popup-title';title.textContent=text(stop.name);root.appendChild(title);if(stop.address){const address=document.createElement('p');address.className='popup-meta';address.textContent=text(stop.address);root.appendChild(address)}if(Array.isArray(stop.routes)&&stop.routes.length){const list=document.createElement('div');list.className='route-list';list.setAttribute('aria-label',labels.routes);stop.routes.forEach(route=>{const badge=document.createElement('span');badge.className='route-badge';badge.textContent=text(route.number);if(route.palette&&validColor(route.palette.background)&&validColor(route.palette.foreground)){badge.style.backgroundColor=route.palette.background;badge.style.color=route.palette.foreground}list.appendChild(badge)});root.appendChild(list)}return root}
function updateSelection(){markers.forEach((marker,id)=>{const element=marker.getElement();if(!element)return;const selected=id===selectedStopId;element.classList.toggle('selected',selected);element.setAttribute('aria-selected',String(selected))})}
function setStops(stops){markers.forEach(marker=>marker.remove());markers.clear();if(!Array.isArray(stops))return;stops.forEach(stop=>{if(!stop||typeof stop.id!=='string'||!finite(stop.latitude)||!finite(stop.longitude))return;const marker=L.marker([stop.latitude,stop.longitude],{icon:L.divIcon({className:'stop-marker',html:'',iconSize:[24,24],iconAnchor:[12,12]}),title:text(stop.name),alt:labels.stop+': '+text(stop.name)});marker.bindPopup(popupFor(stop));marker.on('click',()=>{selectedStopId=stop.id;updateSelection();send({type:'stopSelected',stopId:stop.id})});marker.addTo(map);markers.set(stop.id,marker)});updateSelection()}
function receive(event){try{const message=JSON.parse(event.data);if(!message||typeof message.type!=='string')return;if(message.type==='initialize'){applyTheme(message.tokens);document.documentElement.lang=typeof message.language==='string'?message.language:'en';if(message.labels&&typeof message.labels==='object')labels={...labels,...message.labels};document.getElementById('map').setAttribute('aria-label',labels.map)}else if(message.type==='setStops')setStops(message.stops);else if(message.type==='setUserLocation'&&finite(message.latitude)&&finite(message.longitude)){if(userMarker)userMarker.setLatLng([message.latitude,message.longitude]);else userMarker=L.marker([message.latitude,message.longitude],{icon:L.divIcon({className:'user-marker',html:'',iconSize:[20,20],iconAnchor:[10,10]}),title:labels.userLocation,alt:labels.userLocation}).addTo(map)}else if(message.type==='setView'&&finite(message.latitude)&&finite(message.longitude))map.setView([message.latitude,message.longitude],Number.isFinite(message.zoom)?message.zoom:15);else if(message.type==='centerOnUser'&&userMarker)map.setView(userMarker.getLatLng(),16)}catch(_error){send({type:'bridgeError'})}}
document.addEventListener('message',receive);window.addEventListener('message',receive);send({type:'mapReady'});
}());
</script></body></html>`;

export function MapScreen({ navigation }: MapScreenProps) {
  const { t, i18n } = useTranslation();
  const { colors, theme, webViewTokens } = useTheme();
  const webViewRef = useRef<WebView>(null);
  const [stops, setStops] = useState<Stop[]>([]);
  const [routes, setRoutes] = useState<Route[]>([]);
  const [status, setStatus] = useState<TransitDataState>('loading');
  const [lastUpdatedAt, setLastUpdatedAt] = useState<string | null>(null);
  const [viewMode, setViewMode] = useState<'map' | 'list'>('map');
  const [selectedRoute, setSelectedRoute] = useState<string | null>(null);
  const [selectedStop, setSelectedStop] = useState<string | null>(null);
  const [mapReady, setMapReady] = useState(false);
  const [mapUnavailable, setMapUnavailable] = useState(false);
  const [locationStatus, setLocationStatus] = useState<'idle' | 'requesting' | 'granted' | 'denied' | 'unavailable'>('idle');
  const [userLocation, setUserLocation] = useState<{ latitude: number; longitude: number } | null>(null);
  const [locationCheckedAt, setLocationCheckedAt] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [searchResults, setSearchResults] = useState<Stop[]>([]);
  const [searchStatus, setSearchStatus] = useState<'idle' | 'loading' | 'success' | 'empty' | 'error' | 'offline'>('idle');

  const linePalette = useCallback((candidate: unknown) => getAccessibleLineColor(candidate, colors.primaryContainer, colors.text, colors.onInverse, colors.onPrimaryContainer), [colors]);
  const visibleStops = useMemo(() => {
    const source = searchStatus === 'success' || searchStatus === 'empty' ? searchResults : stops;
    return selectedRoute ? source.filter(stop => stop.routes?.some(route => (route.id ?? route.route_id) === selectedRoute)) : source;
  }, [searchResults, searchStatus, selectedRoute, stops]);
  const sendToMap = useCallback((message: MapBridgeMessage) => webViewRef.current?.postMessage(serializeWebViewMessage(message)), []);

  const loadData = useCallback(async (refresh = false) => {
    const hasData = stops.length > 0 || routes.length > 0;
    setStatus(refresh && hasData ? 'refreshing' : 'loading');
    const [stopsResult, routesResult] = await Promise.allSettled([apiClient.get('/stops'), apiClient.get('/routes')]);
    const failed = [stopsResult, routesResult].filter(result => result.status === 'rejected');
    const offline = failed.some(result => result.status === 'rejected' && axios.isAxiosError(result.reason) && !result.reason.response);
    const nextStops = stopsResult.status === 'fulfilled' ? asArray<Stop>(stopsResult.value.data, 'stops') : stops;
    const nextRoutes = routesResult.status === 'fulfilled' ? asArray<Route>(routesResult.value.data, 'routes') : routes;
    if (stopsResult.status === 'fulfilled') setStops(nextStops);
    if (routesResult.status === 'fulfilled') setRoutes(nextRoutes);
    setStatus(classifyTransitState({ hasData: nextStops.length > 0 || nextRoutes.length > 0, offline, partial: failed.length === 1, failed: failed.length === 2 }));
    if (failed.length === 0) setLastUpdatedAt(new Date().toISOString());
  }, [routes, stops]);

  const requestLocation = useCallback(async () => {
    setLocationStatus('requesting');
    try {
      const permission = await Location.requestForegroundPermissionsAsync();
      if (permission.status !== 'granted') { setLocationStatus('denied'); return; }
      const location = await Location.getCurrentPositionAsync({ accuracy: Location.Accuracy.Balanced });
      const next = { latitude: location.coords.latitude, longitude: location.coords.longitude };
      setUserLocation(next); setLocationStatus('granted');
      setLocationCheckedAt(new Date(Number.isFinite(location.timestamp) ? location.timestamp : Date.now()).toISOString());
      sendToMap({ type: 'setUserLocation', ...next }); sendToMap({ type: 'setView', ...next, zoom: 15 });
    } catch (_error) { setLocationStatus('unavailable'); }
  }, [sendToMap]);

  useEffect(() => { void loadData(); }, []);
  useEffect(() => { void requestLocation(); }, []);
  useEffect(() => {
    if (status === 'loading' || viewMode !== 'map' || mapReady || mapUnavailable) return;
    const timeout = setTimeout(() => setMapUnavailable(true), 12000);
    return () => clearTimeout(timeout);
  }, [mapReady, mapUnavailable, status, viewMode]);
  useEffect(() => {
    if (!mapReady) return;
    sendToMap({ type: 'initialize', tokens: webViewTokens, language: i18n.language, labels: { map: t('transit_map.accessibility_label'), stop: t('transit_map.stop_marker'), userLocation: t('transit_map.user_location'), routes: t('transit_map.routes_label') } });
    sendToMap({ type: 'setStops', stops: visibleStops.map(stop => ({ id: stop.id, name: stop.name, address: stop.address ?? '', latitude: stop.latitude, longitude: stop.longitude, routes: (stop.routes ?? []).map(route => ({ number: route.route_number, palette: linePalette(route.route_color) })) })) });
    if (userLocation) sendToMap({ type: 'setUserLocation', ...userLocation });
  }, [i18n.language, linePalette, mapReady, sendToMap, t, userLocation, visibleStops, webViewTokens]);

  const handleSearch = useCallback(async () => {
    const query = searchQuery.trim();
    if (!query) { setSearchResults([]); setSearchStatus('idle'); return; }
    setSearchStatus('loading');
    try {
      const response = await apiClient.post('/stops/search', { query, limit: 25 });
      const results = asArray<Stop>(response.data, 'stops');
      setSearchResults(results); setSearchStatus(results.length ? 'success' : 'empty');
    } catch (error) { setSearchStatus(axios.isAxiosError(error) && !error.response ? 'offline' : 'error'); }
  }, [searchQuery]);

  const handleMapMessage = useCallback((event: { nativeEvent: { data: string } }) => {
    try {
      const value: unknown = JSON.parse(event.nativeEvent.data);
      if (!value || typeof value !== 'object') return;
      const message = value as Record<string, unknown>;
      if (message.type === 'mapReady') { setMapReady(true); setMapUnavailable(false); }
      else if (message.type === 'mapUnavailable' || message.type === 'bridgeError') setMapUnavailable(true);
      else if (message.type === 'stopSelected' && typeof message.stopId === 'string' && visibleStops.some(stop => stop.id === message.stopId)) setSelectedStop(message.stopId);
    } catch (_error) { setMapUnavailable(true); }
  }, [visibleStops]);

  const allowMapNavigation = useCallback((request: { url: string }) => {
    const kind = mapNavigationKind(request.url);
    if (kind === 'internal') return true;
    if (kind === 'attribution') void Linking.openURL(request.url);
    return false;
  }, []);

  const updatedAt = lastUpdatedAt ? new Intl.DateTimeFormat(i18n.language === 'pl' ? 'pl-PL' : 'en-GB', { dateStyle: 'short', timeStyle: 'short' }).format(new Date(lastUpdatedAt)) : t('transit_common.not_available');
  const locationReadAt = locationCheckedAt ? new Intl.DateTimeFormat(i18n.language === 'pl' ? 'pl-PL' : 'en-GB', { dateStyle: 'short', timeStyle: 'short' }).format(new Date(locationCheckedAt)) : null;
  const stateBadge = status === 'offline' ? <StatusBadge label={t('transit_common.offline')} tone="offline" /> : status === 'stale' ? <StatusBadge label={t('transit_common.stale')} tone="stale" /> : status === 'partial' ? <StatusBadge label={t('transit_common.partial')} tone="warning" /> : status === 'refreshing' ? <StatusBadge label={t('transit_common.refreshing')} tone="info" /> : null;

  if (status === 'loading') return <Screen accessibilityLabel={t('transit_map.title')} contentContainerStyle={styles.centered}><StatusBadge label={t('transit_common.loading')} tone="info" /></Screen>;
  if ((status === 'error' || status === 'offline') && stops.length === 0) return <Screen accessibilityLabel={t('transit_map.title')} contentContainerStyle={styles.centered}><ErrorState description={t(status === 'offline' ? 'transit_common.offline_description' : 'transit_map.load_error_description')} onRetry={() => void loadData()} title={t(status === 'offline' ? 'transit_common.offline' : 'transit_map.load_error_title')} /></Screen>;

  return (
    <Screen accessibilityLabel={t('transit_map.title')} padded={false}>
      <View style={[styles.controls, { borderColor: colors.border, gap: theme.space.md, padding: theme.space.lg }]}>
        <AppText accessibilityRole="header" variant="h1">{t('transit_map.title')}</AppText>
        <TextField label={t('transit_map.search_label')} onChangeText={setSearchQuery} onSubmitEditing={() => void handleSearch()} placeholder={t('transit_map.search_placeholder')} returnKeyType="search" value={searchQuery} />
        <View style={styles.searchActions}>
          <Button icon="search-outline" label={t('transit_map.search_action')} loading={searchStatus === 'loading'} loadingLabel={t('transit_map.searching')} onPress={() => void handleSearch()} variant="secondary" />
          {searchQuery ? <IconButton accessibilityLabel={t('transit_map.clear_search')} icon="close-outline" onPress={() => { setSearchQuery(''); setSearchResults([]); setSearchStatus('idle'); }} /> : null}
          <IconButton accessibilityLabel={t('transit_common.refresh')} icon="refresh-outline" onPress={() => void loadData(true)} />
        </View>
        {searchStatus === 'empty' ? <AppText tone="secondary">{t('transit_map.search_empty')}</AppText> : null}
        {searchStatus === 'error' || searchStatus === 'offline' ? <AppText accessibilityLiveRegion="polite" tone="warning">{t(searchStatus === 'offline' ? 'transit_common.offline_description' : 'transit_map.search_error')}</AppText> : null}
        {locationStatus === 'denied' || locationStatus === 'unavailable' ? <Card style={{ backgroundColor: colors.warningContainer }}><AppText style={{ color: colors.onWarningContainer }} variant="bodyStrong">{t(locationStatus === 'denied' ? 'transit_map.permission_denied' : 'transit_map.location_unavailable')}</AppText><AppText style={{ color: colors.onWarningContainer }}>{t('transit_map.manual_search_hint')}</AppText><Button label={t('transit_map.retry_location')} onPress={() => void requestLocation()} variant="quiet" /></Card> : null}
        <View accessibilityRole="tablist" style={[styles.segmented, { borderColor: colors.borderStrong, borderRadius: theme.radius.md }]}>
          {(['map', 'list'] as const).map(mode => <Pressable accessibilityRole="tab" accessibilityState={{ selected: viewMode === mode }} key={mode} onPress={() => { if (mode === 'list') setMapReady(false); setViewMode(mode); }} style={[styles.segment, { minHeight: theme.size.touch }, viewMode === mode && { backgroundColor: colors.primaryContainer }]}><Ionicons accessibilityElementsHidden color={viewMode === mode ? colors.onPrimaryContainer : colors.textSecondary} name={mode === 'map' ? 'map-outline' : 'list-outline'} size={20} /><AppText style={{ color: viewMode === mode ? colors.onPrimaryContainer : colors.textSecondary }} variant="label">{t(`transit_map.view_${mode}`)}</AppText></Pressable>)}
        </View>
        <FlatList accessibilityLabel={t('transit_map.route_filter')} data={[null, ...routes]} horizontal keyExtractor={item => item?.id ?? 'all'} renderItem={({ item }) => { const selected = selectedRoute === (item?.id ?? null); const palette = item ? linePalette(item.color) : null; return <Pressable accessibilityRole="button" accessibilityState={{ selected }} onPress={() => setSelectedRoute(item?.id ?? null)} style={[styles.routeChip, { backgroundColor: selected ? colors.primaryContainer : colors.surface, borderColor: selected ? colors.primary : colors.borderStrong, borderRadius: theme.radius.md, minHeight: theme.size.touch }]}>{palette ? <View style={[styles.lineSwatch, { backgroundColor: palette.background }]} /> : null}<AppText style={{ color: selected ? colors.onPrimaryContainer : colors.text }} variant="label">{item?.number ?? t('transit_map.all_routes')}</AppText></Pressable>; }} showsHorizontalScrollIndicator={false} />
        <View style={styles.metaRow}><AppText tone="secondary" variant="caption">{t('transit_common.source')}: {t('transit_map.source_value')}</AppText><AppText tone="secondary" variant="caption">{t('transit_common.retrieved_at')}: {updatedAt}</AppText>{locationReadAt ? <AppText tone="secondary" variant="caption">{t('transit_map.location_read_at')}: {locationReadAt}</AppText> : null}{stateBadge}</View>
      </View>
      {viewMode === 'map' ? <View style={styles.mapContainer}>
        <WebView accessibilityLabel={t('transit_map.accessibility_label')} domStorageEnabled={false} javaScriptEnabled mixedContentMode="never" onError={() => setMapUnavailable(true)} onHttpError={() => setMapUnavailable(true)} onMessage={handleMapMessage} onShouldStartLoadWithRequest={allowMapNavigation} originWhitelist={['about:blank', 'https://app.roadrunner.invalid']} ref={webViewRef} source={{ html: MAP_HTML_TEMPLATE, baseUrl: 'https://app.roadrunner.invalid/' }} style={styles.map} />
        {mapUnavailable ? <View style={[styles.mapFallback, { backgroundColor: colors.surface }]}><ErrorState description={t('transit_map.map_unavailable_description')} onRetry={() => { setMapReady(false); setMapUnavailable(false); webViewRef.current?.reload(); }} title={t('transit_map.map_unavailable_title')} /><Button label={t('transit_map.open_list')} onPress={() => { setMapReady(false); setViewMode('list'); }} variant="secondary" /></View> : null}
        <IconButton accessibilityLabel={t('transit_map.center_user')} disabled={!userLocation} icon="locate-outline" onPress={() => sendToMap({ type: 'centerOnUser' })} style={[styles.locationButton, { backgroundColor: colors.surface, borderColor: colors.borderStrong }]} />
      </View> : visibleStops.length === 0 ? <EmptyState description={t('transit_map.empty_description')} icon="bus-outline" style={styles.listState} title={t('transit_map.empty_title')} /> : <FlatList contentContainerStyle={[styles.stopList, { padding: theme.space.lg }]} data={visibleStops} keyExtractor={item => item.id} renderItem={({ item }) => <Card accessibilityLabel={t('transit_map.open_stop', { name: item.name })} onPress={() => { setSelectedStop(item.id); navigation.navigate('StopDetails', { stopId: item.id }); }} style={selectedStop === item.id ? { backgroundColor: colors.primaryContainer, borderColor: colors.primary } : undefined}><View style={styles.stopRow}><Ionicons accessibilityElementsHidden color={colors.primary} name="location-outline" size={theme.size.icon} /><View style={styles.stopText}><AppText variant="bodyStrong">{item.name}</AppText>{item.address ? <AppText tone="secondary" variant="caption">{item.address}</AppText> : null}</View><Ionicons accessibilityElementsHidden color={colors.textSecondary} name="chevron-forward-outline" size={theme.size.icon} /></View></Card>} />}
      {viewMode === 'map' && selectedStop && visibleStops.some(stop => stop.id === selectedStop) ? <View style={{ padding: theme.space.lg }}><Card accessibilityLabel={t('transit_map.open_stop', { name: visibleStops.find(stop => stop.id === selectedStop)?.name ?? '' })} onPress={() => navigation.navigate('StopDetails', { stopId: selectedStop })}><AppText variant="bodyStrong">{visibleStops.find(stop => stop.id === selectedStop)?.name ?? ''}</AppText><AppText tone="secondary">{t('transit_map.open_stop', { name: visibleStops.find(stop => stop.id === selectedStop)?.name ?? '' })}</AppText></Card></View> : null}
    </Screen>
  );
}

export default MapScreen;

const styles = StyleSheet.create({
  centered: { alignItems: 'center', flex: 1, justifyContent: 'center' }, controls: { borderBottomWidth: 1 }, lineSwatch: { borderRadius: 4, height: 16, width: 16 }, listState: { flex: 1 },
  locationButton: { borderWidth: 1, bottom: 16, position: 'absolute', right: 16 }, map: { flex: 1 }, mapContainer: { flex: 1, minHeight: 280, position: 'relative' }, mapFallback: { ...StyleSheet.absoluteFillObject, alignItems: 'center', justifyContent: 'center', padding: 16 },
  metaRow: { alignItems: 'flex-start', gap: 4 }, routeChip: { alignItems: 'center', borderWidth: 1, flexDirection: 'row', gap: 8, justifyContent: 'center', marginRight: 8, paddingHorizontal: 12 }, searchActions: { alignItems: 'center', flexDirection: 'row', gap: 8 },
  segment: { alignItems: 'center', flex: 1, flexDirection: 'row', gap: 8, justifyContent: 'center' }, segmented: { borderWidth: 1, flexDirection: 'row', overflow: 'hidden' }, stopList: { gap: 12, paddingBottom: 32 }, stopRow: { alignItems: 'center', flexDirection: 'row', gap: 12 }, stopText: { flex: 1 },
});
