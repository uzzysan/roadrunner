/**
 * MapScreen - Ekran mapy z przystankami
 * 
 * Wyświetla mapę OpenStreetMap z zaznaczonymi przystankami.
 * Umożliwia:
 * - Przeglądanie przystanków na mapie
 * - Wyszukiwanie najbliższego przystanku
 * - Podgląd szczegółów przystanku
 * - Filtrowanie przystanków według linii
 */

import React, { useState, useEffect, useCallback, useRef } from 'react';
import {
  View,
  StyleSheet,
  Dimensions,
  ActivityIndicator,
  TouchableOpacity,
  TextInput,
  FlatList,
  Alert,
  Platform,
} from 'react-native';
import { WebView } from 'react-native-webview';
import * as Location from 'expo-location';
import { useTranslation } from 'react-i18next';
import { Ionicons } from '@expo/vector-icons';

import { ThemedView } from '../components/ThemedView';
import { ThemedText } from '../components/ThemedText';
import { Card } from '../components/Card';
import { Button } from '../components/Button';
import { useTheme } from '../hooks/useTheme';
import { api } from '../services/api';
import { Stop, Route } from '../types';

const { width, height } = Dimensions.get('window');

// Szablon HTML dla OpenStreetMap z Leaflet
const MAP_HTML_TEMPLATE = `
<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no" />
  <link rel="stylesheet" href="https://unpkg.com/leaflet@1.9.4/dist/leaflet.css" />
  <script src="https://unpkg.com/leaflet@1.9.4/dist/leaflet.js"></script>
  <style>
    body { margin: 0; padding: 0; }
    #map { height: 100vh; width: 100vw; }
    .stop-popup { 
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
      min-width: 150px;
    }
    .stop-popup h3 { 
      margin: 0 0 8px 0; 
      font-size: 14px; 
      font-weight: 600;
    }
    .stop-popup p { 
      margin: 0 0 4px 0; 
      font-size: 12px; 
      color: #666;
    }
    .stop-popup .routes {
      margin-top: 8px;
      display: flex;
      flex-wrap: wrap;
      gap: 4px;
    }
    .stop-popup .route-badge {
      background: #2563EB;
      color: white;
      padding: 2px 8px;
      border-radius: 12px;
      font-size: 11px;
      font-weight: 600;
    }
    .custom-marker {
      background: #2563EB;
      border: 3px solid white;
      border-radius: 50%;
      width: 24px;
      height: 24px;
      box-shadow: 0 2px 6px rgba(0,0,0,0.3);
    }
    .custom-marker.selected {
      background: #EF4444;
      width: 32px;
      height: 32px;
    }
    .user-location-marker {
      background: #10B981;
      border: 3px solid white;
      border-radius: 50%;
      width: 20px;
      height: 20px;
      box-shadow: 0 2px 6px rgba(0,0,0,0.3);
      animation: pulse 2s infinite;
    }
    @keyframes pulse {
      0% { transform: scale(1); opacity: 1; }
      50% { transform: scale(1.2); opacity: 0.8; }
      100% { transform: scale(1); opacity: 1; }
    }
  </style>
</head>
<body>
  <div id="map"></div>
  <script>
    // Inicjalizacja mapy
    const map = L.map('map', {
      zoomControl: false,
      attributionControl: false
    });

    // Dodanie warstwy OpenStreetMap
    L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
      maxZoom: 19,
      attribution: '© OpenStreetMap contributors'
    }).addTo(map);

    // Przechowywanie markerów
    const markers = {};
    let userLocationMarker = null;
    let selectedStopId = null;

    // Funkcja do ustawiania pozycji mapy
    function setView(lat, lng, zoom) {
      map.setView([lat, lng], zoom);
    }

    // Funkcja do dodawania przystanku
    function addStop(stop) {
      const marker = L.marker([stop.latitude, stop.longitude], {
        icon: L.divIcon({
          className: 'custom-marker',
          html: '<div></div>',
          iconSize: [24, 24],
          iconAnchor: [12, 12]
        })
      });

      const popupContent = \`
        <div class="stop-popup">
          <h3>\${stop.name}</h3>
          <p>\${stop.address || ''}</p>
          \${stop.routes ? `<div class="routes">\${stop.routes.map(r => 
            `<span class="route-badge" style="background: \${r.color}">\${r.number}</span>`
          ).join('')}</div>` : ''}
        </div>
      \`;

      marker.bindPopup(popupContent);
      marker.on('click', function() {
        selectedStopId = stop.id;
        window.ReactNativeWebView.postMessage(JSON.stringify({
          type: 'stopSelected',
          stopId: stop.id
        }));
        updateMarkerStyles();
      });

      marker.addTo(map);
      markers[stop.id] = marker;
    }

    // Funkcja do aktualizacji stylów markerów
    function updateMarkerStyles() {
      Object.keys(markers).forEach(id => {
        const marker = markers[id];
        const element = marker.getElement();
        if (element) {
          const div = element.querySelector('.custom-marker');
          if (div) {
            if (id === selectedStopId) {
              div.classList.add('selected');
            } else {
              div.classList.remove('selected');
            }
          }
        }
      });
    }

    // Funkcja do czyszczenia wszystkich markerów
    function clearMarkers() {
      Object.values(markers).forEach(marker => marker.remove());
      Object.keys(markers).forEach(key => delete markers[key]);
    }

    // Funkcja do ustawiania lokalizacji użytkownika
    function setUserLocation(lat, lng) {
      if (userLocationMarker) {
        userLocationMarker.setLatLng([lat, lng]);
      } else {
        userLocationMarker = L.marker([lat, lng], {
          icon: L.divIcon({
            className: 'user-location-marker',
            html: '<div></div>',
            iconSize: [20, 20],
            iconAnchor: [10, 10]
          })
        }).addTo(map);
      }
    }

    // Funkcja do wyśrodkowania na lokalizacji użytkownika
    function centerOnUser() {
      if (userLocationMarker) {
        const latLng = userLocationMarker.getLatLng();
        map.setView(latLng, 16);
      }
    }

    // Nasłuchiwanie wiadomości z React Native
    document.addEventListener('message', function(event) {
      const data = JSON.parse(event.data);
      
      switch(data.type) {
        case 'setView':
          setView(data.lat, data.lng, data.zoom);
          break;
        case 'addStop':
          addStop(data.stop);
          break;
        case 'clearMarkers':
          clearMarkers();
          break;
        case 'setUserLocation':
          setUserLocation(data.lat, data.lng);
          break;
        case 'centerOnUser':
          centerOnUser();
          break;
      }
    });

    // Domyślna lokalizacja (Warszawa)
    setView(52.2297, 21.0122, 12);
  </script>
</body>
</html>
`;

interface MapScreenProps {
  navigation: any;
}

export default function MapScreen({ navigation }: MapScreenProps) {
  const { t } = useTranslation();
  const { theme, colors } = useTheme();
  const webViewRef = useRef<WebView>(null);

  const [stops, setStops] = useState<Stop[]>([]);
  const [routes, setRoutes] = useState<Route[]>([]);
  const [loading, setLoading] = useState(true);
  const [userLocation, setUserLocation] = useState<{ lat: number; lng: number } | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [showSearchResults, setShowSearchResults] = useState(false);
  const [nearbyStops, setNearbyStops] = useState<Stop[]>([]);
  const [selectedRoute, setSelectedRoute] = useState<string | null>(null);
  const [mapReady, setMapReady] = useState(false);

  // Pobierz przystanki i linie przy starcie
  useEffect(() => {
    loadData();
    requestLocationPermission();
  }, []);

  // Aktualizuj markery na mapie gdy dane się zmienią
  useEffect(() => {
    if (mapReady && stops.length > 0) {
      updateMapMarkers();
    }
  }, [stops, mapReady]);

  const loadData = async () => {
    try {
      setLoading(true);
      const [stopsResponse, routesResponse] = await Promise.all([
        api.get('/stops'),
        api.get('/routes'),
      ]);
      setStops(stopsResponse.data.stops);
      setRoutes(routesResponse.data.routes);
    } catch (error) {
      console.error('Error loading data:', error);
      Alert.alert(t('common.error'), t('map.loadError'));
    } finally {
      setLoading(false);
    }
  };

  const requestLocationPermission = async () => {
    try {
      const { status } = await Location.requestForegroundPermissionsAsync();
      if (status === 'granted') {
        const location = await Location.getCurrentPositionAsync({
          accuracy: Location.Accuracy.Balanced,
        });
        const { latitude, longitude } = location.coords;
        setUserLocation({ lat: latitude, lng: longitude });
        
        // Wyślij lokalizację do mapy
        if (webViewRef.current) {
          webViewRef.current.postMessage(JSON.stringify({
            type: 'setUserLocation',
            lat: latitude,
            lng: longitude,
          }));
          webViewRef.current.postMessage(JSON.stringify({
            type: 'setView',
            lat: latitude,
            lng: longitude,
            zoom: 15,
          }));
        }

        // Pobierz pobliskie przystanki
        loadNearbyStops(latitude, longitude);
      }
    } catch (error) {
      console.error('Error getting location:', error);
    }
  };

  const loadNearbyStops = async (lat: number, lng: number) => {
    try {
      const response = await api.get('/stops/nearby', {
        params: { lat, lon: lng, radius: 1000 },
      });
      setNearbyStops(response.data.map((item: any) => item.stop));
    } catch (error) {
      console.error('Error loading nearby stops:', error);
    }
  };

  const updateMapMarkers = () => {
    if (!webViewRef.current) return;

    // Wyczyść istniejące markery
    webViewRef.current.postMessage(JSON.stringify({ type: 'clearMarkers' }));

    // Dodaj markery dla przystanków
    const filteredStops = selectedRoute
      ? stops.filter(stop => stop.routes?.some(r => r.id === selectedRoute))
      : stops;

    filteredStops.forEach(stop => {
      webViewRef.current?.postMessage(JSON.stringify({
        type: 'addStop',
        stop: {
          id: stop.id,
          name: stop.name,
          latitude: stop.latitude,
          longitude: stop.longitude,
          address: stop.address,
          routes: stop.routes,
        },
      }));
    });
  };

  const handleMessage = useCallback((event: any) => {
    try {
      const data = JSON.parse(event.nativeEvent.data);
      
      if (data.type === 'stopSelected') {
        const stop = stops.find(s => s.id === data.stopId);
        if (stop) {
          navigation.navigate('StopDetails', { stopId: stop.id });
        }
      }
    } catch (error) {
      console.error('Error handling message:', error);
    }
  }, [stops, navigation]);

  const handleSearch = async () => {
    if (!searchQuery.trim()) return;

    try {
      const response = await api.post('/stops/search', {
        query: searchQuery,
        limit: 10,
      });
      setShowSearchResults(true);
    } catch (error) {
      console.error('Error searching stops:', error);
    }
  };

  const centerOnUser = () => {
    if (webViewRef.current && userLocation) {
      webViewRef.current.postMessage(JSON.stringify({
        type: 'centerOnUser',
      }));
    }
  };

  const renderRouteFilter = () => (
    <View style={styles.routeFilter}>
      <FlatList
        horizontal
        showsHorizontalScrollIndicator={false}
        data={[{ id: null, name: t('map.allRoutes'), color: '#666' }, ...routes]}
        keyExtractor={(item) => item.id || 'all'}
        renderItem={({ item }) => (
          <TouchableOpacity
            style={[
              styles.routeChip,
              {
                backgroundColor: selectedRoute === item.id
                  ? item.color || '#2563EB'
                  : colors.card,
                borderColor: item.color || '#2563EB',
              },
            ]}
            onPress={() => {
              setSelectedRoute(item.id === selectedRoute ? null : (item.id as string));
            }}
          >
            <ThemedText
              style={[
                styles.routeChipText,
                {
                  color: selectedRoute === item.id
                    ? '#fff'
                    : colors.text,
                },
              ]}
            >
              {item.number || item.name}
            </ThemedText>
          </TouchableOpacity>
        )}
      />
    </View>
  );

  if (loading) {
    return (
      <ThemedView style={styles.loadingContainer}>
        <ActivityIndicator size="large" color={colors.primary} />
        <ThemedText style={styles.loadingText}>{t('common.loading')}</ThemedText>
      </ThemedView>
    );
  }

  return (
    <ThemedView style={styles.container}>
      {/* Pasek wyszukiwania */}
      <View style={[styles.searchBar, { backgroundColor: colors.card }]}>
        <Ionicons name="search" size={20} color={colors.textSecondary} />
        <TextInput
          style={[styles.searchInput, { color: colors.text }]}
          placeholder={t('map.searchPlaceholder')}
          placeholderTextColor={colors.textSecondary}
          value={searchQuery}
          onChangeText={setSearchQuery}
          onSubmitEditing={handleSearch}
        />
        {searchQuery.length > 0 && (
          <TouchableOpacity onPress={() => setSearchQuery('')}>
            <Ionicons name="close-circle" size={20} color={colors.textSecondary} />
          </TouchableOpacity>
        )}
      </View>

      {/* Filtr linii */}
      {renderRouteFilter()}

      {/* Mapa */}
      <View style={styles.mapContainer}>
        <WebView
          ref={webViewRef}
          originWhitelist={['*']}
          source={{ html: MAP_HTML_TEMPLATE }}
          style={styles.map}
          onMessage={handleMessage}
          onLoad={() => setMapReady(true)}
          javaScriptEnabled={true}
          domStorageEnabled={true}
          geolocationEnabled={true}
        />

        {/* Przycisk lokalizacji */}
        <TouchableOpacity
          style={[styles.locationButton, { backgroundColor: colors.card }]}
          onPress={centerOnUser}
        >
          <Ionicons name="locate" size={24} color={colors.primary} />
        </TouchableOpacity>
      </View>

      {/* Lista pobliskich przystanków */}
      {nearbyStops.length > 0 && (
        <View style={[styles.nearbyContainer, { backgroundColor: colors.card }]}>
          <ThemedText style={styles.nearbyTitle}>
            {t('map.nearbyStops')}
          </ThemedText>
          <FlatList
            horizontal
            showsHorizontalScrollIndicator={false}
            data={nearbyStops.slice(0, 5)}
            keyExtractor={(item) => item.id}
            renderItem={({ item }) => (
              <TouchableOpacity
                style={styles.nearbyItem}
                onPress={() => {
                  if (webViewRef.current) {
                    webViewRef.current.postMessage(JSON.stringify({
                      type: 'setView',
                      lat: item.latitude,
                      lng: item.longitude,
                      zoom: 17,
                    }));
                  }
                }}
              >
                <Ionicons name="location" size={16} color={colors.primary} />
                <ThemedText style={styles.nearbyItemText} numberOfLines={1}>
                  {item.name}
                </ThemedText>
              </TouchableOpacity>
            )}
          />
        </View>
      )}
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
  loadingText: {
    marginTop: 16,
    fontSize: 16,
  },
  searchBar: {
    flexDirection: 'row',
    alignItems: 'center',
    margin: 12,
    paddingHorizontal: 12,
    paddingVertical: 8,
    borderRadius: 12,
    ...Platform.select({
      ios: {
        shadowColor: '#000',
        shadowOffset: { width: 0, height: 2 },
        shadowOpacity: 0.1,
        shadowRadius: 4,
      },
      android: {
        elevation: 4,
      },
    }),
  },
  searchInput: {
    flex: 1,
    marginLeft: 8,
    fontSize: 16,
    paddingVertical: 4,
  },
  routeFilter: {
    paddingHorizontal: 12,
    marginBottom: 8,
  },
  routeChip: {
    paddingHorizontal: 12,
    paddingVertical: 6,
    borderRadius: 16,
    marginRight: 8,
    borderWidth: 1,
  },
  routeChipText: {
    fontSize: 12,
    fontWeight: '600',
  },
  mapContainer: {
    flex: 1,
    position: 'relative',
  },
  map: {
    flex: 1,
  },
  locationButton: {
    position: 'absolute',
    right: 16,
    bottom: 16,
    width: 48,
    height: 48,
    borderRadius: 24,
    justifyContent: 'center',
    alignItems: 'center',
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
  nearbyContainer: {
    padding: 12,
    borderTopLeftRadius: 16,
    borderTopRightRadius: 16,
  },
  nearbyTitle: {
    fontSize: 14,
    fontWeight: '600',
    marginBottom: 8,
  },
  nearbyItem: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: 'rgba(0,0,0,0.05)',
    paddingHorizontal: 12,
    paddingVertical: 8,
    borderRadius: 20,
    marginRight: 8,
  },
  nearbyItemText: {
    fontSize: 13,
    marginLeft: 4,
    maxWidth: 120,
  },
});
