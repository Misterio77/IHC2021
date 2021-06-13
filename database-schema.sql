SET statement_timeout = 0;

SET lock_timeout = 0;

SET idle_in_transaction_session_timeout = 0;

SET client_encoding = 'UTF8';

SET standard_conforming_strings = ON;

SELECT
    pg_catalog.set_config('search_path', '', FALSE);

SET check_function_bodies = FALSE;

SET xmloption = content;

SET client_min_messages = warning;

SET row_security = OFF;

CREATE EXTENSION IF NOT EXISTS citext WITH SCHEMA public;

COMMENT ON EXTENSION citext IS 'data type for case-insensitive character strings';

CREATE EXTENSION IF NOT EXISTS "uuid-ossp" WITH SCHEMA public;

COMMENT ON EXTENSION "uuid-ossp" IS 'generate universally unique identifiers (UUIDs)';

CREATE DOMAIN public.email AS public.citext CONSTRAINT email_check CHECK ((VALUE OPERATOR (public. ~) '^[a-zA-Z0-9.!#$%&''*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$'::public.citext));

ALTER DOMAIN public.email OWNER TO misterio;

SET default_tablespace = '';

SET default_table_access_method = heap;

CREATE TABLE public.products (
    slug text NOT NULL,
    shop_slug text NOT NULL,
    name text NOT NULL,
    price numeric NOT NULL,
    available integer DEFAULT 0,
    sold integer DEFAULT 0 NOT NULL,
    details text,
    picture text
);

ALTER TABLE public.products OWNER TO misterio;

COMMENT ON COLUMN public.products.available IS 'NULL means "unlimited"';

CREATE SEQUENCE public.products_id_seq
    AS integer START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER TABLE public.products_id_seq OWNER TO misterio;

ALTER SEQUENCE public.products_id_seq OWNED BY public.products.slug;

CREATE TABLE public.purchases (
    id integer NOT NULL,
    product_slug text,
    amount integer NOT NULL,
    paid numeric NOT NULL,
    purchaser_email public.citext,
    "time" timestamp with time zone NOT NULL
);

ALTER TABLE public.purchases OWNER TO misterio;

COMMENT ON COLUMN public.purchases.product_slug IS 'Nullable to keep record even for deleted products';

COMMENT ON COLUMN public.purchases.purchaser_email IS 'Nullable to keep record even if user is deleted';

CREATE TABLE public.sessions (
    id integer NOT NULL,
    token text NOT NULL,
    created timestamp with time zone DEFAULT now() NOT NULL,
    user_email public.citext NOT NULL,
    used timestamp with time zone
);

ALTER TABLE public.sessions OWNER TO misterio;

CREATE SEQUENCE public.sessions_id_seq
    AS integer START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER TABLE public.sessions_id_seq OWNER TO misterio;

ALTER SEQUENCE public.sessions_id_seq OWNED BY public.sessions.id;

CREATE TABLE public.shops (
    slug text NOT NULL,
    name text NOT NULL,
    color character varying(6),
    owner_email public.citext NOT NULL
);

ALTER TABLE public.shops OWNER TO misterio;

COMMENT ON COLUMN public.shops.slug IS 'Shop slug name';

CREATE TABLE public.users (
    email public.citext NOT NULL,
    password text NOT NULL,
    name text NOT NULL
);

ALTER TABLE public.users OWNER TO misterio;

COMMENT ON COLUMN public.users.email IS 'User email';

ALTER TABLE ONLY public.products
    ALTER COLUMN slug SET DEFAULT nextval('public.products_id_seq'::regclass);

ALTER TABLE ONLY public.sessions
    ALTER COLUMN id SET DEFAULT nextval('public.sessions_id_seq'::regclass);

ALTER TABLE ONLY public.products
    ADD CONSTRAINT products_pkey PRIMARY KEY (slug);

ALTER TABLE ONLY public.sessions
    ADD CONSTRAINT sessions_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.shops
    ADD CONSTRAINT shop_pkey PRIMARY KEY (slug);

ALTER TABLE ONLY public.sessions
    ADD CONSTRAINT unique_token UNIQUE (token);

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (email);

ALTER TABLE ONLY public.products
    ADD CONSTRAINT products_shop_slug_fkey FOREIGN KEY (shop_slug) REFERENCES public.shops (slug) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.purchases
    ADD CONSTRAINT sales_product_slug_fkey FOREIGN KEY (product_slug) REFERENCES public.products (slug) ON UPDATE CASCADE ON DELETE SET NULL;

ALTER TABLE ONLY public.purchases
    ADD CONSTRAINT sales_purchaser_email_fkey FOREIGN KEY (purchaser_email) REFERENCES public.users (email) ON UPDATE CASCADE ON DELETE SET NULL;

ALTER TABLE ONLY public.sessions
    ADD CONSTRAINT sessions_user_email_fkey FOREIGN KEY (user_email) REFERENCES public.users (email) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.shops
    ADD CONSTRAINT shops_owner_email_fkey FOREIGN KEY (owner_email) REFERENCES public.users (email) ON UPDATE CASCADE ON DELETE CASCADE;

