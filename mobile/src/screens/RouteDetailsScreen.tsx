/**
 * RouteDetailsScreen - Ekran szczegółów linii autobusowej
 * 
 * Wyświetla:
 * - Informacje o linii (numer, nazwa, kolor)
 * - Listę przystanków w kolejności trasy
 * - Mapę z trasą
 * - Pełny rozkład jazdy
 */

import React, { useState, useEffect, useRef } from 'react';
import {
  View,
  StyleSheet,
  ScrollView,
  ActivityIndicator,
  TouchableOpacity,
  FlatList,
} from 'react-native';
import { WebView } from 'react-native-webview';
import { useTranslation } from 'react-i18next';
import { Ionicons } from '@expo/vector-icons';

import { ThemedView } from '../components/ThemedView';
import { ThemedText } from '../components/ThemedText';
import { Card } from '../components/Card';
import { Button } from '../components/Button';
import { useTheme } from '../hooks/useTheme';
import { api } from '../services/api';
import { Route, StopInRoute, RouteSchedule } from '../types';

// Szablon HTML dla mapy z trasą
const ROUTE_MAP_HTML = `
<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <link rel="stylesheet" href="https://unpkg.com/leaflet@1.9.4/dist/leaflet.css" />
  <script src="https://unpkg.com/leaflet@1.9.4/dist/leaflet.js"></script>
  <style>
    body { margin: 0; padding: 0; }
    #map { height: 100vh; width: 100vw; }
    .stop-marker {
      background: white;
      border: 3px solid {ROUTE_COLOR};
      border-radius: 50%;
      width: 24px;
      height: 24px;
      display: flex;
      align-items: center;
      justify-content: center;
      font-size: 10px;
      font-weight: 700;
      color: {ROUTE_COLOR};
      box-shadow: 0 2px 6px rgba(0,0,0,0.3);
    }
    .stop-marker.start {
      background: #10B981;
      border-color: #10B981;
      color: white;
    }
    .stop-marker.end {
      background: #EF4444;
      border-color: #EF4444;
      color: white;
    }
  </style>
</head>
<body>
  <div id="map"></div>
  <script>
    const map = L.map('map', {
      zoomControl: false,
      attributionControl: false,
    });

    L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
      maxZoom: 19,
    }).addTo(map);

    const markers = [];
    let polyline = null;

    function clearMap() {
      markers.forEach(m => m.remove());
      markers.length = 0;
      if (polyline) {
        polyline.remove();
        polyline = null;
      }
    }

    function setRoute(data) {
      clearMap();
      
      const coordinates = data.coordinates;
      if (coordinates.length === 0) return;

      // Dodaj markery dla przystanków
      coordinates.forEach((coord, index) => {
        const isFirst = index === 0;
        const isLast = index === coordinates.length - 1;
        
        const marker = L.marker([coord.lat, coord.lon], {
          icon: L.divIcon({
            className: \`stop-marker \${isFirst ? 'start' : ''} \${isLast ? 'end' : ''}\`,
            html: \`<div>\${isFirst ? 'A' : isLast ? 'B' : index + 1}</div>\`,
            iconSize: [24, 24],
            iconAnchor: [12, 12]
          })
        });

        marker.bindPopup(\`<b>\${coord.stop_name}</b><br>Przystanek \${index + 1}\`);
        marker.addTo(map);
        markers.push(marker);
      });

      // Narysuj linię trasy
      const latLngs = coordinates.map(c => [c.lat, c.lon]);
      polyline = L.polyline(latLngs, {
        color: data.routeColor,
        weight: 4,
        opacity: 0.8,
      }).addTo(map);

      // Dopasuj widok do trasy
      map.fitBounds(polyline.getBounds(), { padding: [30, 30] });
    }

    document.addEventListener('message', function(event) {
      const data = JSON.parse(event.data);
      if (data.type === 'setRoute') {
        setRoute(data);
      }
    });
  </script>
</body>
</html>
`;

interface RouteDetailsScreenProps {
  navigation: any;
  route: { params: { routeId: string } };
}

interface StopSchedule {
  stopId: string;
  stopName: string;
  stopOrder: number;
  weekdayDepartures: string[];
  saturdayDepartures: string[];
  sundayDepartures: string[];
}

export default function RouteDetailsScreen({ navigation, route: navRoute }: RouteDetailsScreenProps) {
  const { t } = useTranslation();
  const { colors } = useTheme();
  const webViewRef = useRef<WebView>(null);
  const { routeId } = navRoute.params;

  const [route, setRoute] = useState<Route | null>(null);
  const [stops, setStops] = useState<StopInRoute[]>([]);
  const [schedules, setSchedules] = useState<StopSchedule[]>([]);
  const [loading, setLoading] = useState(true);
  const [activeTab, setActiveTab] = useState<'stops' | 'schedule'>('stops');
  const [expandedDayType, setExpandedDayType] = useState<string>('weekday');
  const [selectedStop, setSelectedStop] = useState<string | null>(null);

  useEffect(() => {
    loadRouteData();
  }, [routeId]);

  useEffect(() => {
    if (stops.length > 0 && route && webViewRef.current) {
      const mapHtml = ROUTE_MAP_HTML.replace(/{ROUTE_COLOR}/g, route.color);
      
      // Wyślij dane trasy do mapy
      setTimeout(() => {
        webViewRef.current?.postMessage(JSON.stringify({
          type: 'setRoute',
          routeColor: route.color,
          coordinates: stops.map(s => ({
            lat: s.latitude,
            lon: s.longitude,
            stop_name: s.name,
          })),
        }));
      }, 500);
    }
  }, [stops, route]);

  const loadRouteData = async () => {
    try {
      setLoading(true);
      const [routeResponse, schedulesResponse] = await Promise.all([
        api.get(`/routes/${routeId}`),
        api.get(`/routes/${routeId}/schedules`),
      ]);

      setRoute(routeResponse.data.route);
      setStops(routeResponse.data.stops);
      setSchedules(schedulesResponse.data.schedules_by_stop);
    } catch (error) {
      console.error('Error loading route data:', error);
    } finally {
      setLoading(false);
    }
  };

  const renderStopsList = () => (
    <View style={styles.stopsList}>
      {stops.map((stop, index) => (
        <TouchableOpacity
          key={stop.id}
          style={[
            styles.stopItem,
            selectedStop === stop.id && { backgroundColor: route?.color + '15' },
          ]}
          onPress={() => {
            setSelectedStop(stop.id === selectedStop ? null : stop.id);
            navigation.navigate('StopDetails', { stopId: stop.id });
          }}
        >
          {/* Numer przystanku */}
          <View style={[styles.stopNumber, { backgroundColor: route?.color }]}>
            <ThemedText style={styles.stopNumberText}>{index + 1}</ThemedText>
          </View>

          {/* Linia łącząca */}
          {index < stops.length - 1 && (
            <View style={[styles.connector, { backgroundColor: route?.color + '40' }]} />
          )}

          {/* Informacje o przystanku */}
          <View style={styles.stopInfo}>
            <ThemedText style={styles.stopName}>{stop.name}</ThemedText>
            {stop.is_optional && (
              <View style={styles.optionalBadge}>
                <ThemedText style={styles.optionalText}>
                  {t('routeDetails.onRequest')}
                </ThemedText>
              </View>
            )}
          </View>

          <Ionicons name="chevron-forward" size={20} color={colors.textSecondary} />
        </TouchableOpacity>
      ))}
    </View>
  );

  const renderSchedule = () => {
    const filteredSchedules = selectedStop
      ? schedules.filter(s => s.stop_id === selectedStop)
      : schedules;

    if (filteredSchedules.length === 0) {
      return (
        <View style={styles.emptySchedule}>
          <ThemedText style={styles.emptyScheduleText}>
            {t('routeDetails.noSchedule')}
          </ThemedText>
        </View>
      );
    }

    return (
      <View>
        {/* Filtr przystanków */}
        <ScrollView
          horizontal
          showsHorizontalScrollIndicator={false}
          style={styles.stopFilter}
        >
          <TouchableOpacity
            style={[
              styles.stopFilterChip,
              !selectedStop && styles.stopFilterChipActive,
              { borderColor: route?.color },
            ]}
            onPress={() => setSelectedStop(null)}
          >
            <ThemedText
              style={[
                styles.stopFilterText,
                !selectedStop && { color: route?.color },
              ]}
            >
              {t('routeDetails.allStops')}
            </ThemedText>
          </TouchableOpacity>
          {stops.map(stop => (
            <TouchableOpacity
              key={stop.id}
              style={[
                styles.stopFilterChip,
                selectedStop === stop.id && [
                  styles.stopFilterChipActive,
                  { backgroundColor: route?.color + '20' },
                ],
                { borderColor: route?.color },
              ]}
              onPress={() => setSelectedStop(stop.id === selectedStop ? null : stop.id)}
            >
              <ThemedText
                style={[
                  styles.stopFilterText,
                  selectedStop === stop.id && { color: route?.color },
                ]}
                numberOfLines={1}
              >
                {stop.stop_order}. {stop.name}
              </ThemedText>
            </TouchableOpacity>
          ))}
        </ScrollView>

        {/* Zakładki dni */}
        <View style={styles.dayTabs}>
          {['weekday', 'saturday', 'sunday'].map(day => (
            <TouchableOpacity
              key={day}
              style={[
                styles.dayTab,
                expandedDayType === day && [
                  styles.dayTabActive,
                  { borderBottomColor: route?.color },
                ],
              ]}
              onPress={() => setExpandedDayType(day)}
            >
              <ThemedText
                style={[
                  styles.dayTabText,
                  expandedDayType === day && [
                    styles.dayTabTextActive,
                    { color: route?.color },
                  ],
                ]}
              >
                {t(`dayTypes.${day}`)}
              </ThemedText>
            </TouchableOpacity>
          ))}
        </View>

        {/* Rozkład dla każdego przystanku */}
        {filteredSchedules.map(schedule => (
          <Card key={schedule.stop_id} style={styles.scheduleCard}>
            <ThemedText style={styles.scheduleStopName}>
              {schedule.stop_order}. {schedule.stop_name}
            </ThemedText>

            <View style={styles.departuresContainer}>
              {(expandedDayType === 'weekday' ? schedule.weekday_departures :
                expandedDayType === 'saturday' ? schedule.saturday_departures :
                  schedule.sunday_departures
              ).map((time, idx) => (
                <View
                  key={idx}
                  style={[styles.departureTime, { backgroundColor: route?.color + '15' }]}
                >
                  <ThemedText style={[styles.departureTimeText, { color: route?.color }]}>
                    {time}
                  </ThemedText>
                </View>
              ))}
            </View>
          </Card>
        ))}
      </View>
    );
  };

  if (loading) {
    return (
      <ThemedView style={styles.loadingContainer}>
        <ActivityIndicator size="large" color={colors.primary} />
      </ThemedView>
    );
  }

  if (!route) {
    return (
      <ThemedView style={styles.errorContainer}>
        <Ionicons name="alert-circle" size={64} color={colors.error} />
        <ThemedText style={styles.errorText}>{t('routeDetails.notFound')}</ThemedText>
      </ThemedView>
    );
  }

  return (
    <ThemedView style={styles.container}>
      {/* Nagłówek z mapą */}
      <View style={styles.header}>
        <WebView
          ref={webViewRef}
          originWhitelist={['*']}
          source={{ html: ROUTE_MAP_HTML.replace(/{ROUTE_COLOR}/g, route.color) }}
          style={styles.map}
        />

        {/* Informacje o linii */}
        <View style={[styles.routeInfo, { backgroundColor: colors.card }]}>
          <View style={[styles.routeBadge, { backgroundColor: route.color }]}>
            <ThemedText style={styles.routeNumber}>{route.number}</ThemedText>
          </View>
          <View style={styles.routeTextInfo}>
            <ThemedText style={styles.routeName}>{route.name}</ThemedText>
            <ThemedText style={[styles.routeDescription, { color: colors.textSecondary }]}>
              {route.description}
            </ThemedText>
          </View>
        </View>
      </View>

      {/* Zakładki */}
      <View style={[styles.tabs, { backgroundColor: colors.card }]}>
        <TouchableOpacity
          style={[styles.tab, activeTab === 'stops' && styles.tabActive]}
          onPress={() => setActiveTab('stops')}
        >
          <Ionicons
            name="location"
            size={18}
            color={activeTab === 'stops' ? route.color : colors.textSecondary}
          />
          <ThemedText
            style={[
              styles.tabText,
              { color: activeTab === 'stops' ? route.color : colors.textSecondary },
            ]}
          >
            {t('routeDetails.stops')} ({stops.length})
          </ThemedText>
        </TouchableOpacity>

        <TouchableOpacity
          style={[styles.tab, activeTab === 'schedule' && styles.tabActive]}
          onPress={() => setActiveTab('schedule')}
        >
          <Ionicons
            name="time"
            size={18}
            color={activeTab === 'schedule' ? route.color : colors.textSecondary}
          />
          <ThemedText
            style={[
              styles.tabText,
              { color: activeTab === 'schedule' ? route.color : colors.textSecondary },
            ]}
          >
            {t('routeDetails.schedule')}
          </ThemedText>
        </TouchableOpacity>
      </View>

      {/* Zawartość */}
      <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
        {activeTab === 'stops' ? renderStopsList() : renderSchedule()}
      </ScrollView>
    </ThemedView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  loadingContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
  },
  errorContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    padding: 24,
  },
  errorText: {
    marginTop: 16,
    fontSize: 16,
    textAlign: 'center',
  },
  header: {
    height: 280,
  },
  map: {
    flex: 1,
  },
  routeInfo: {
    flexDirection: 'row',
    alignItems: 'center',
    padding: 16,
  },
  routeBadge: {
    width: 56,
    height: 56,
    borderRadius: 28,
    justifyContent: 'center',
    alignItems: 'center',
  },
  routeNumber: {
    color: 'white',
    fontSize: 20,
    fontWeight: '700',
  },
  routeTextInfo: {
    flex: 1,
    marginLeft: 16,
  },
  routeName: {
    fontSize: 18,
    fontWeight: '700',
  },
  routeDescription: {
    fontSize: 14,
    marginTop: 2,
  },
  tabs: {
    flexDirection: 'row',
    borderBottomWidth: 1,
    borderBottomColor: 'rgba(0,0,0,0.1)',
  },
  tab: {
    flex: 1,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    paddingVertical: 14,
  },
  tabActive: {
    borderBottomWidth: 2,
  },
  tabText: {
    fontSize: 14,
    fontWeight: '600',
    marginLeft: 8,
  },
  content: {
    flex: 1,
    padding: 16,
  },
  stopsList: {
    paddingLeft: 8,
  },
  stopItem: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingVertical: 12,
    paddingHorizontal: 8,
    borderRadius: 12,
  },
  stopNumber: {
    width: 32,
    height: 32,
    borderRadius: 16,
    justifyContent: 'center',
    alignItems: 'center',
    zIndex: 1,
  },
  stopNumberText: {
    color: 'white',
    fontSize: 12,
    fontWeight: '700',
  },
  connector: {
    position: 'absolute',
    left: 23,
    top: 36,
    width: 2,
    height: 32,
  },
  stopInfo: {
    flex: 1,
    marginLeft: 16,
    flexDirection: 'row',
    alignItems: 'center',
  },
  stopName: {
    fontSize: 15,
    fontWeight: '500',
  },
  optionalBadge: {
    backgroundColor: 'rgba(0,0,0,0.05)',
    paddingHorizontal: 8,
    paddingVertical: 2,
    borderRadius: 10,
    marginLeft: 8,
  },
  optionalText: {
    fontSize: 10,
    opacity: 0.7,
  },
  stopFilter: {
    marginBottom: 16,
  },
  stopFilterChip: {
    paddingHorizontal: 14,
    paddingVertical: 8,
    borderRadius: 20,
    borderWidth: 1,
    marginRight: 8,
    maxWidth: 200,
  },
  stopFilterChipActive: {
    borderWidth: 0,
  },
  stopFilterText: {
    fontSize: 13,
    fontWeight: '500',
  },
  dayTabs: {
    flexDirection: 'row',
    marginBottom: 16,
  },
  dayTab: {
    flex: 1,
    paddingVertical: 12,
    alignItems: 'center',
    borderBottomWidth: 2,
    borderBottomColor: 'transparent',
  },
  dayTabActive: {
    borderBottomWidth: 2,
  },
  dayTabText: {
    fontSize: 13,
    fontWeight: '500',
  },
  dayTabTextActive: {
    fontWeight: '600',
  },
  scheduleCard: {
    marginBottom: 12,
  },
  scheduleStopName: {
    fontSize: 15,
    fontWeight: '600',
    marginBottom: 12,
  },
  departuresContainer: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
  },
  departureTime: {
    paddingHorizontal: 12,
    paddingVertical: 8,
    borderRadius: 8,
    minWidth: 56,
    alignItems: 'center',
  },
  departureTimeText: {
    fontSize: 14,
    fontWeight: '600',
    fontVariant: ['tabular-nums'],
  },
  emptySchedule: {
    padding: 40,
    alignItems: 'center',
  },
  emptyScheduleText: {
    fontSize: 14,
    opacity: 0.6,
  },
});
