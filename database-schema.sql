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

CREATE DOMAIN public.email AS public.citext CONSTRAINT email_check CHECK ((VALUE OPERATOR (public. ~) '^[a-zA-Z0-9.!#$%&''*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$'::public.citext));

CREATE TABLE public.products (
    slug text NOT NULL,
    shop text NOT NULL,
    name text NOT NULL,
    price numeric NOT NULL,
    available integer NOT NULL,
    sold integer NOT NULL,
    details text NOT NULL,
    picture text NOT NULL
);

CREATE TABLE public.purchases (
    product text,
    amount integer NOT NULL,
    paid numeric NOT NULL,
    purchaser public.citext,
    "time" timestamp with time zone NOT NULL
);

COMMENT ON COLUMN public.purchases.product IS 'Nullable to keep record even for deleted products';

COMMENT ON COLUMN public.purchases.purchaser IS 'Nullable to keep record even if user is deleted';

CREATE SEQUENCE public.purchases_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

CREATE TABLE public.shops (
    slug text NOT NULL,
    name text NOT NULL,
    color_dark character varying(6) NOT NULL,
    color_light character varying(6) NOT NULL,
    manager public.citext NOT NULL,
    logo text NOT NULL
);

COMMENT ON COLUMN public.shops.slug IS 'Shop slug name';

CREATE TABLE public.users (
    email public.citext NOT NULL,
    password text NOT NULL,
    name text NOT NULL,
    admin boolean NOT NULL,
    token text DEFAULT 'NULL' ::text
);

COMMENT ON COLUMN public.users.email IS 'User email';

ALTER TABLE ONLY public.products
    ADD CONSTRAINT products_pkey PRIMARY KEY (slug);

ALTER TABLE ONLY public.purchases
    ADD CONSTRAINT purchases_time PRIMARY KEY ("time");

ALTER TABLE ONLY public.shops
    ADD CONSTRAINT shop_pkey PRIMARY KEY (slug);

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (email);

ALTER TABLE ONLY public.products
    ADD CONSTRAINT products_shop_slug_fkey FOREIGN KEY (shop) REFERENCES public.shops (slug) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.purchases
    ADD CONSTRAINT sales_product_slug_fkey FOREIGN KEY (product) REFERENCES public.products (slug) ON UPDATE CASCADE ON DELETE SET NULL;

ALTER TABLE ONLY public.purchases
    ADD CONSTRAINT sales_purchaser_email_fkey FOREIGN KEY (purchaser) REFERENCES public.users (email) ON UPDATE CASCADE ON DELETE SET NULL;

ALTER TABLE ONLY public.shops
    ADD CONSTRAINT shops_owner_email_fkey FOREIGN KEY (manager) REFERENCES public.users (email) ON UPDATE CASCADE ON DELETE CASCADE;

