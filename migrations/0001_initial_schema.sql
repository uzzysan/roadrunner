-- Initial schema for RoadRunner
-- Users table with role-based access

CREATE TYPE user_role AS ENUM ('passenger', 'driver', 'attendant', 'parent', 'admin');

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    email_hash VARCHAR(64) NOT NULL, -- For lookups without exposing email
    password_hash VARCHAR(255) NOT NULL,
    first_name VARCHAR(100) NOT NULL,
    last_name VARCHAR(100) NOT NULL,
    phone VARCHAR(20),
    role user_role NOT NULL DEFAULT 'passenger',
    mfa_enabled BOOLEAN NOT NULL DEFAULT false,
    mfa_secret VARCHAR(255),
    email_verified BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Students table for school module
CREATE TABLE students (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    first_name VARCHAR(100) NOT NULL,
    last_name VARCHAR(100) NOT NULL,
    student_id VARCHAR(50) UNIQUE, -- School ID
    school_id UUID,
    default_stop_id UUID,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Parent-Student relationships
CREATE TYPE relationship_type AS ENUM ('mother', 'father', 'legal_guardian', 'other');

CREATE TABLE parent_student_links (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    student_id UUID NOT NULL REFERENCES students(id) ON DELETE CASCADE,
    relationship_type relationship_type NOT NULL DEFAULT 'other',
    is_primary BOOLEAN NOT NULL DEFAULT false,
    can_receive_alerts BOOLEAN NOT NULL DEFAULT true,
    invite_token VARCHAR(255),
    invite_expires_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(parent_id, student_id)
);

-- Indexes
CREATE INDEX idx_users_email_hash ON users(email_hash);
CREATE INDEX idx_users_role ON users(role);
CREATE INDEX idx_parent_student_parent ON parent_student_links(parent_id);
CREATE INDEX idx_parent_student_student ON parent_student_links(student_id);

-- Update trigger for updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
