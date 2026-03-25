/**
 * TypeScript types dla RoadRunner Mobile App
 */

// ==================== MODELE PODSTAWOWE ====================

export interface User {
  id: string;
  email: string;
  first_name: string;
  last_name: string;
  role: 'passenger' | 'driver' | 'admin';
  is_active: boolean;
  mfa_enabled: boolean;
  created_at: string;
}

export interface Stop {
  id: string;
  name: string;
  description?: string;
  latitude: number;
  longitude: number;
  address?: string;
  amenities?: string[];
  is_active: boolean;
  created_at: string;
  routes?: RouteAtStop[];
}

export interface Route {
  id: string;
  name: string;
  number: string;
  description: string;
  color: string;
  is_active: boolean;
  created_at: string;
}

export interface RouteAtStop {
  route_id: string;
  route_name: string;
  route_number: string;
  route_color: string;
  first_departure?: string;
  last_departure?: string;
}

export interface StopInRoute {
  id: string;
  name: string;
  longitude: number;
  latitude: number;
  stop_order: number;
  is_optional: boolean;
}

export type DayType = 'weekday' | 'saturday' | 'sunday' | 'holiday' | 'everyday';

export interface Schedule {
  id: string;
  route_id: string;
  stop_id: string;
  arrival_time: string;
  departure_time: string;
  day_type: DayType;
  is_active: boolean;
  created_at: string;
}

export interface ScheduleWithRoute extends Schedule {
  route_name: string;
  route_number: string;
  route_color: string;
  route_description?: string;
}

export interface ScheduleWithStop extends Schedule {
  stop_name: string;
  stop_latitude: number;
  stop_longitude: number;
}

export interface RouteSchedule {
  stop_id: string;
  stop_name: string;
  stop_order: number;
  weekday_departures: string[];
  saturday_departures: string[];
  sunday_departures: string[];
}

// ==================== BILETY ====================

export type TicketType = 'single' | 'time' | 'period';
export type TicketStatus = 'active' | 'used' | 'expired' | 'cancelled';

export interface Ticket {
  id: string;
  user_id: string;
  ticket_type: TicketType;
  status: TicketStatus;
  valid_from?: string;
  valid_until?: string;
  used_at?: string;
  qr_code: string;
  price: number;
  currency: string;
  created_at: string;
}

export interface TicketTypeInfo {
  id: string;
  name: string;
  description: string;
  type: TicketType;
  price: number;
  currency: string;
  duration_minutes?: number;
  validity_days?: number;
  is_active: boolean;
}

// ==================== AUTH ====================

export interface LoginRequest {
  email: string;
  password: string;
  mfa_code?: string;
}

export interface LoginResponse {
  access_token: string;
  refresh_token: string;
  token_type: string;
  expires_in: number;
  user: User;
  mfa_required: boolean;
}

export interface RegisterRequest {
  email: string;
  password: string;
  first_name: string;
  last_name: string;
}

export interface RefreshTokenRequest {
  refresh_token: string;
}

// ==================== API RESPONSES ====================

export interface ApiResponse<T> {
  data: T;
  message?: string;
}

export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  per_page: number;
  total_pages: number;
}

export interface StopsListResponse {
  stops: Stop[];
  total: number;
}

export interface RoutesListResponse {
  routes: Route[];
  total: number;
}

export interface NearbyStopResponse {
  stop: Stop;
  distance_meters: number;
}

export interface NextDeparture {
  schedule_id: string;
  departure_time: string;
  route_id: string;
  route_name: string;
  route_number: string;
  route_color: string;
  stop_id: string;
  stop_name: string;
  minutes_until_departure: number;
}

export interface TodaySchedulesResponse {
  date: string;
  day_type: string;
  day_name: string;
  departures_by_route: RouteTodayDepartures[];
}

export interface RouteTodayDepartures {
  route_id: string;
  route_name: string;
  route_number: string;
  route_color: string;
  departures: TodayDeparture[];
}

export interface TodayDeparture {
  schedule_id: string;
  stop_id: string;
  stop_name: string;
  departure_time: string;
  is_past: boolean;
}

// ==================== THEME ====================

export interface ThemeColors {
  primary: string;
  primaryDark: string;
  secondary: string;
  accent: string;
  background: string;
  surface: string;
  card: string;
  text: string;
  textSecondary: string;
  textTertiary: string;
  border: string;
  success: string;
  warning: string;
  error: string;
  info: string;
  disabled: string;
  overlay: string;
}

export type ThemeMode = 'light' | 'dark' | 'system';

// ==================== NAVIGATION ====================

export type RootStackParamList = {
  Login: undefined;
  Register: undefined;
  MfaSetup: { qrCode: string; secret: string };
  MfaVerify: { email: string };
  MainTabs: undefined;
  StopDetails: { stopId: string };
  RouteDetails: { routeId: string };
  TicketDetails: { ticketId: string };
  BuyTicket: undefined;
  QRScanner: undefined;
  Profile: undefined;
  Settings: undefined;
};

export type MainTabParamList = {
  Map: undefined;
  Routes: undefined;
  Tickets: undefined;
  Profile: undefined;
};

// ==================== I18N ====================

export type Language = 'pl' | 'en';

export interface TranslationKeys {
  // Common
  'common.loading': string;
  'common.error': string;
  'common.retry': string;
  'common.cancel': string;
  'common.save': string;
  'common.delete': string;
  'common.edit': string;
  'common.close': string;
  'common.confirm': string;
  'common.next': string;
  'common.back': string;
  'common.search': string;
  'common.filter': string;
  'common.sort': string;
  'common.all': string;
  'common.none': string;
  'common.or': string;
  'common.and': string;

  // Auth
  'auth.login': string;
  'auth.logout': string;
  'auth.register': string;
  'auth.email': string;
  'auth.password': string;
  'auth.confirmPassword': string;
  'auth.forgotPassword': string;
  'auth.resetPassword': string;
  'auth.firstName': string;
  'auth.lastName': string;
  'auth.mfaTitle': string;
  'auth.mfaDescription': string;
  'auth.mfaCode': string;
  'auth.mfaScanQR': string;
  'auth.mfaEnterCode': string;
  'auth.mfaSetupSuccess': string;

  // Map
  'map.title': string;
  'map.searchPlaceholder': string;
  'map.nearbyStops': string;
  'map.allRoutes': string;
  'map.loadError': string;
  'map.noStops': string;
  'map.directions': string;

  // Stop Details
  'stopDetails.title': string;
  'stopDetails.servingRoutes': string;
  'stopDetails.schedule': string;
  'stopDetails.noSchedule': string;
  'stopDetails.notFound': string;
  'stopDetails.directions': string;
  'stopDetails.amenities': string;

  // Route Details
  'routeDetails.title': string;
  'routeDetails.stops': string;
  'routeDetails.schedule': string;
  'routeDetails.allStops': string;
  'routeDetails.onRequest': string;
  'routeDetails.noSchedule': string;
  'routeDetails.notFound': string;

  // Tickets
  'tickets.title': string;
  'tickets.myTickets': string;
  'tickets.buyTicket': string;
  'tickets.active': string;
  'tickets.used': string;
  'tickets.expired': string;
  'tickets.noTickets': string;
  'tickets.ticketTypes': string;
  'tickets.validUntil': string;
  'tickets.validFrom': string;
  'tickets.showQR': string;
  'tickets.scanQR': string;

  // Day Types
  'dayTypes.weekday': string;
  'dayTypes.saturday': string;
  'dayTypes.sunday': string;
  'dayTypes.holiday': string;
  'dayTypes.everyday': string;

  // Amenities
  'amenities.shelter': string;
  'amenities.bench': string;
  'amenities.lighting': string;
  'amenities.monitoring': string;
  'amenities.ticket_machine': string;
  'amenities.accessibility': string;
  'amenities.display': string;

  // Profile
  'profile.title': string;
  'profile.personalInfo': string;
  'profile.changePassword': string;
  'profile.language': string;
  'profile.theme': string;
  'profile.notifications': string;
  'profile.mfa': string;

  // Settings
  'settings.title': string;
  'settings.darkMode': string;
  'settings.lightMode': string;
  'settings.systemMode': string;
  'settings.polish': string;
  'settings.english': string;
}

// ==================== ERRORS ====================

export interface ApiError {
  message: string;
  code?: string;
  status?: number;
  details?: Record<string, string[]>;
}

// ==================== UTILS ====================

export interface Coordinates {
  latitude: number;
  longitude: number;
}

export interface BoundingBox {
  minLat: number;
  maxLat: number;
  minLon: number;
  maxLon: number;
}
