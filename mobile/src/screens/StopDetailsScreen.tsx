/**
 * StopDetailsScreen - Ekran szczegółów przystanku
 * 
 * Wyświetla:
 * - Informacje o przystanku (nazwa, adres, udogodnienia)
 * - Listę linii obsługujących przystanek
 * - Rozkład jazdy dla każdej linii
 * - Mapę z lokalizacją przystanku
 */

import React, { useState, useEffect, useRef } from 'react';
import {
  View,
  StyleSheet,
  ScrollView,
  ActivityIndicator,
  TouchableOpacity,
  FlatList,
  Linking,
  Platform,
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
import { Stop, RouteAtStop, ScheduleWithRoute } from '../types';

// Szablon HTML dla mini-mapy
const MINI_MAP_HTML = `
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
      background: #2563EB;
      border: 4px solid white;
      border-radius: 50%;
      width: 32px;
      height: 32px;
      box-shadow: 0 4px 12px rgba(0,0,0,0.4);
    }
  </style>
</head>
<body>
  <div id="map"></div>
  <script>
    const map = L.map('map', {
      zoomControl: false,
      attributionControl: false,
      dragging: false,
      scrollWheelZoom: false,
      doubleClickZoom: false,
    });

    L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
      maxZoom: 19,
    }).addTo(map);

    document.addEventListener('message', function(event) {
      const data = JSON.parse(event.data);
      if (data.type === 'setStop') {
        map.setView([data.lat, data.lng], 16);
        L.marker([data.lat, data.lng], {
          icon: L.divIcon({
            className: 'stop-marker',
            html: '<div></div>',
            iconSize: [32, 32],
            iconAnchor: [16, 16]
          })
        }).addTo(map);
      }
    });
  </script>
</body>
</html>
`;

interface StopDetailsScreenProps {
  navigation: any;
  route: { params: { stopId: string } };
}

interface GroupedSchedule {
  routeId: string;
  routeName: string;
  routeNumber: string;
  routeColor: string;
  weekdayDepartures: string[];
  saturdayDepartures: string[];
  sundayDepartures: string[];
}

export default function StopDetailsScreen({ navigation, route }: StopDetailsScreenProps) {
  const { t } = useTranslation();
  const { colors } = useTheme();
  const webViewRef = useRef<WebView>(null);
  const { stopId } = route.params;

  const [stop, setStop] = useState<Stop | null>(null);
  const [routes, setRoutes] = useState<RouteAtStop[]>([]);
  const [schedules, setSchedules] = useState<ScheduleWithRoute[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedRoute, setSelectedRoute] = useState<string | null>(null);
  const [expandedDayType, setExpandedDayType] = useState<string>('weekday');

  useEffect(() => {
    loadStopData();
  }, [stopId]);

  useEffect(() => {
    if (stop && webViewRef.current) {
      webViewRef.current.postMessage(JSON.stringify({
        type: 'setStop',
        lat: stop.latitude,
        lng: stop.longitude,
      }));
    }
  }, [stop]);

  const loadStopData = async () => {
    try {
      setLoading(true);
      const [stopResponse, routesResponse, schedulesResponse] = await Promise.all([
        api.get(`/stops/${stopId}`),
        api.get(`/stops/${stopId}/routes`),
        api.get(`/stops/${stopId}/schedules`),
      ]);

      setStop(stopResponse.data);
      setRoutes(routesResponse.data);
      setSchedules(schedulesResponse.data);
    } catch (error) {
      console.error('Error loading stop data:', error);
    } finally {
      setLoading(false);
    }
  };

  const groupSchedulesByRoute = (): GroupedSchedule[] => {
    const grouped: { [key: string]: GroupedSchedule } = {};

    schedules.forEach(schedule => {
      if (!grouped[schedule.route_id]) {
        grouped[schedule.route_id] = {
          routeId: schedule.route_id,
          routeName: schedule.route_name,
          routeNumber: schedule.route_number,
          routeColor: schedule.route_color,
          weekdayDepartures: [],
          saturdayDepartures: [],
          sundayDepartures: [],
        };
      }

      const time = schedule.departure_time.substring(0, 5); // HH:MM
      
      switch (schedule.day_type) {
        case 'weekday':
        case 'everyday':
          if (!grouped[schedule.route_id].weekdayDepartures.includes(time)) {
            grouped[schedule.route_id].weekdayDepartures.push(time);
          }
          break;
        case 'saturday':
          if (!grouped[schedule.route_id].saturdayDepartures.includes(time)) {
            grouped[schedule.route_id].saturdayDepartures.push(time);
          }
          break;
        case 'sunday':
        case 'holiday':
          if (!grouped[schedule.route_id].sundayDepartures.includes(time)) {
            grouped[schedule.route_id].sundayDepartures.push(time);
          }
          break;
      }
    });

    // Sortuj odjazdy
    Object.values(grouped).forEach(group => {
      group.weekdayDepartures.sort();
      group.saturdayDepartures.sort();
      group.sundayDepartures.sort();
    });

    return Object.values(grouped);
  };

  const openInMaps = () => {
    if (!stop) return;
    
    const url = Platform.select({
      ios: `maps://?q=${stop.name}&ll=${stop.latitude},${stop.longitude}`,
      android: `geo:${stop.latitude},${stop.longitude}?q=${stop.latitude},${stop.longitude}(${stop.name})`,
    });

    if (url) {
      Linking.openURL(url);
    }
  };

  const renderAmenities = () => {
    if (!stop?.amenities || stop.amenities.length === 0) return null;

    const amenityIcons: { [key: string]: string } = {
      shelter: 'home',
      bench: 'body',
      lighting: 'sunny',
      monitoring: 'videocam',
      ticket_machine: 'card',
      accessibility: 'accessibility',
      display: 'desktop',
    };

    return (
      <View style={styles.amenitiesContainer}>
        {stop.amenities.map((amenity, index) => (
          <View key={index} style={[styles.amenityBadge, { backgroundColor: colors.primary + '20' }]}>
            <Ionicons
              name={amenityIcons[amenity] || 'checkmark-circle'}
              size={14}
              color={colors.primary}
            />
            <ThemedText style={[styles.amenityText, { color: colors.primary }]}>
              {t(`amenities.${amenity}`, amenity)}
            </ThemedText>
          </View>
        ))}
      </View>
    );
  };

  const renderRoutesList = () => (
    <Card style={styles.routesCard}>
      <ThemedText style={styles.sectionTitle}>{t('stopDetails.servingRoutes')}</ThemedText>
      <View style={styles.routesList}>
        {routes.map(route => (
          <TouchableOpacity
            key={route.route_id}
            style={[
              styles.routeItem,
              { backgroundColor: selectedRoute === route.route_id ? route.route_color + '20' : 'transparent' },
            ]}
            onPress={() => setSelectedRoute(
              selectedRoute === route.route_id ? null : route.route_id
            )}
          >
            <View style={[styles.routeBadge, { backgroundColor: route.route_color }]}>
              <ThemedText style={styles.routeBadgeText}>{route.route_number}</ThemedText>
            </View>
            <View style={styles.routeInfo}>
              <ThemedText style={styles.routeName}>{route.route_name}</ThemedText>
              {route.first_departure && route.last_departure && (
                <ThemedText style={styles.routeHours}>
                  {route.first_departure.substring(0, 5)} - {route.last_departure.substring(0, 5)}
                </ThemedText>
              )}
            </View>
            <Ionicons
              name={selectedRoute === route.route_id ? 'chevron-up' : 'chevron-down'}
              size={20}
              color={colors.textSecondary}
            />
          </TouchableOpacity>
        ))}
      </View>
    </Card>
  );

  const renderSchedule = () => {
    const groupedSchedules = groupSchedulesByRoute();
    const filteredSchedules = selectedRoute
      ? groupedSchedules.filter(s => s.routeId === selectedRoute)
      : groupedSchedules;

    if (filteredSchedules.length === 0) {
      return (
        <Card style={styles.scheduleCard}>
          <ThemedText style={styles.noScheduleText}>
            {t('stopDetails.noSchedule')}
          </ThemedText>
        </Card>
      );
    }

    return filteredSchedules.map(group => (
      <Card key={group.routeId} style={styles.scheduleCard}>
        <View style={styles.scheduleHeader}>
          <View style={[styles.routeBadge, { backgroundColor: group.routeColor }]}>
            <ThemedText style={styles.routeBadgeText}>{group.routeNumber}</ThemedText>
          </View>
          <ThemedText style={styles.scheduleRouteName}>{group.routeName}</ThemedText>
        </View>

        {/* Zakładki dni */}
        <View style={styles.dayTabs}>
          {['weekday', 'saturday', 'sunday'].map(day => {
            const hasDepartures = day === 'weekday'
              ? group.weekdayDepartures.length > 0
              : day === 'saturday'
                ? group.saturdayDepartures.length > 0
                : group.sundayDepartures.length > 0;

            return (
              <TouchableOpacity
                key={day}
                style={[
                  styles.dayTab,
                  expandedDayType === day && styles.dayTabActive,
                  !hasDepartures && styles.dayTabDisabled,
                ]}
                onPress={() => hasDepartures && setExpandedDayType(day)}
                disabled={!hasDepartures}
              >
                <ThemedText
                  style={[
                    styles.dayTabText,
                    expandedDayType === day && styles.dayTabTextActive,
                    !hasDepartures && styles.dayTabTextDisabled,
                  ]}
                >
                  {t(`dayTypes.${day}`)}
                </ThemedText>
              </TouchableOpacity>
            );
          })}
        </View>

        {/* Odjazdy */}
        <View style={styles.departuresContainer}>
          {(expandedDayType === 'weekday' ? group.weekdayDepartures :
            expandedDayType === 'saturday' ? group.saturdayDepartures :
              group.sundayDepartures
          ).map((time, index) => (
            <View key={index} style={styles.departureTime}>
              <ThemedText style={styles.departureTimeText}>{time}</ThemedText>
            </View>
          ))}
        </View>
      </Card>
    ));
  };

  if (loading) {
    return (
      <ThemedView style={styles.loadingContainer}>
        <ActivityIndicator size="large" color={colors.primary} />
      </ThemedView>
    );
  }

  if (!stop) {
    return (
      <ThemedView style={styles.errorContainer}>
        <Ionicons name="alert-circle" size={64} color={colors.error} />
        <ThemedText style={styles.errorText}>{t('stopDetails.notFound')}</ThemedText>
      </ThemedView>
    );
  }

  return (
    <ThemedView style={styles.container}>
      <ScrollView showsVerticalScrollIndicator={false}>
        {/* Mini mapa */}
        <View style={styles.mapContainer}>
          <WebView
            ref={webViewRef}
            originWhitelist={['*']}
            source={{ html: MINI_MAP_HTML }}
            style={styles.miniMap}
          />
          <TouchableOpacity
            style={[styles.directionsButton, { backgroundColor: colors.card }]}
            onPress={openInMaps}
          >
            <Ionicons name="navigate" size={20} color={colors.primary} />
            <ThemedText style={[styles.directionsText, { color: colors.primary }]}>
              {t('stopDetails.directions')}
            </ThemedText>
          </TouchableOpacity>
        </View>

        {/* Informacje o przystanku */}
        <View style={styles.infoContainer}>
          <ThemedText style={styles.stopName}>{stop.name}</ThemedText>
          {stop.address && (
            <View style={styles.addressRow}>
              <Ionicons name="location" size={16} color={colors.textSecondary} />
              <ThemedText style={[styles.address, { color: colors.textSecondary }]}>
                {stop.address}
              </ThemedText>
            </View>
          )}
          {renderAmenities()}
        </View>

        {/* Linie obsługujące przystanek */}
        {routes.length > 0 && renderRoutesList()}

        {/* Rozkład jazdy */}
        <View style={styles.scheduleSection}>
          <ThemedText style={styles.sectionTitle}>{t('stopDetails.schedule')}</ThemedText>
          {renderSchedule()}
        </View>
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
  mapContainer: {
    height: 200,
    position: 'relative',
  },
  miniMap: {
    flex: 1,
  },
  directionsButton: {
    position: 'absolute',
    right: 16,
    bottom: 16,
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: 12,
    paddingVertical: 8,
    borderRadius: 20,
    ...Platform.select({
      ios: {
        shadowColor: '#000',
        shadowOffset: { width: 0, height: 2 },
        shadowOpacity: 0.2,
        shadowRadius: 4,
      },
      android: {
        elevation: 4,
      },
    }),
  },
  directionsText: {
    marginLeft: 6,
    fontSize: 13,
    fontWeight: '600',
  },
  infoContainer: {
    padding: 16,
  },
  stopName: {
    fontSize: 24,
    fontWeight: '700',
    marginBottom: 8,
  },
  addressRow: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 12,
  },
  address: {
    fontSize: 14,
    marginLeft: 6,
  },
  amenitiesContainer: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
  },
  amenityBadge: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: 10,
    paddingVertical: 6,
    borderRadius: 16,
  },
  amenityText: {
    fontSize: 12,
    marginLeft: 4,
    fontWeight: '500',
  },
  sectionTitle: {
    fontSize: 18,
    fontWeight: '700',
    marginBottom: 12,
  },
  routesCard: {
    margin: 16,
    marginTop: 0,
  },
  routesList: {
    gap: 8,
  },
  routeItem: {
    flexDirection: 'row',
    alignItems: 'center',
    padding: 12,
    borderRadius: 12,
  },
  routeBadge: {
    width: 44,
    height: 44,
    borderRadius: 22,
    justifyContent: 'center',
    alignItems: 'center',
  },
  routeBadgeText: {
    color: 'white',
    fontSize: 14,
    fontWeight: '700',
  },
  routeInfo: {
    flex: 1,
    marginLeft: 12,
  },
  routeName: {
    fontSize: 15,
    fontWeight: '600',
  },
  routeHours: {
    fontSize: 13,
    marginTop: 2,
    opacity: 0.7,
  },
  scheduleSection: {
    padding: 16,
    paddingTop: 0,
  },
  scheduleCard: {
    marginBottom: 12,
  },
  scheduleHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 16,
  },
  scheduleRouteName: {
    fontSize: 16,
    fontWeight: '600',
    marginLeft: 12,
  },
  dayTabs: {
    flexDirection: 'row',
    marginBottom: 16,
  },
  dayTab: {
    flex: 1,
    paddingVertical: 10,
    alignItems: 'center',
    borderBottomWidth: 2,
    borderBottomColor: 'transparent',
  },
  dayTabActive: {
    borderBottomColor: '#2563EB',
  },
  dayTabDisabled: {
    opacity: 0.4,
  },
  dayTabText: {
    fontSize: 13,
    fontWeight: '500',
  },
  dayTabTextActive: {
    color: '#2563EB',
    fontWeight: '600',
  },
  dayTabTextDisabled: {
    opacity: 0.5,
  },
  departuresContainer: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
  },
  departureTime: {
    backgroundColor: 'rgba(0,0,0,0.05)',
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
  noScheduleText: {
    textAlign: 'center',
    padding: 24,
    opacity: 0.6,
  },
});
