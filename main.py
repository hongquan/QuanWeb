#!/usr/bin/env python

import asyncio
import datetime
import re
import tempfile
from collections.abc import Iterable
from dataclasses import dataclass
from datetime import UTC
from pathlib import Path
from typing import Any, cast
from urllib.parse import urlparse
from uuid import UUID
from zoneinfo import ZoneInfo

import click
import gel
import httpx
import msgspec
from logbook import Logger, StreamHandler
from logbook.more import ColorizedStderrHandler
from safe_result import Err, Ok, Result

from queries.list_post_containing_imgur_async_edgeql import (
    ListPostContainingImgurResult,
    list_post_containing_imgur,
)

# Constants
BUNNY_STORAGE_ZONE = 'quan-images'
BUNNY_STORAGE_DOMAIN = 'sg.storage.bunnycdn.com'
BUNNY_HOST = 'quan-images.b-cdn.net'
log = Logger(__name__)

@dataclass
class MiniBlogPost:
    id: UUID
    body: str

# Type definitions
@dataclass
class ImageEntry:
    imgur: str
    bunny: str | None = None

@dataclass
class PostReport:
    id: UUID
    images: tuple[ImageEntry, ...]

ExtractionResults = list[PostReport]

IMGUR_HEADERS: dict[str, str] = {
    'User-Agent': 'Mozilla/5.0 (X11; Linux x86_64; rv:146.0) Gecko/20100101 Firefox/146.0',
    'Accept': 'image/avif,image/webp,image/png,image/svg+xml,image/*;q=0.8,*/*;q=0.5',
    'Accept-Language': 'vi,en-US;q=0.7,en;q=0.3',
    'Accept-Encoding': 'gzip, deflate, br, zstd',
    'Connection': 'keep-alive',
    'Referer': 'https://quan.hoabinh.vn/',
    'Sec-Fetch-Dest': 'image',
    'Sec-Fetch-Mode': 'no-cors',
    'Sec-Fetch-Site': 'cross-site',
    'Pragma': 'no-cache',
    'Cache-Control': 'no-cache',
}

IMGUR_PATTERN = re.compile(r'https?://i\.imgur\.com/[^\s"\')\]>]+')


@click.group()
def cli() -> None:
    """CLI tool for processing Imgur images in blog posts."""
    pass


@cli.command()
def extract_imgur_images() -> None:
    """Extract Imgur images from blog posts and save to YAML file."""
    asyncio.run(process_extract())


async def download_image(
    http_client: httpx.AsyncClient, url: str, imgur_dir: Path, post_index: int, total_posts: int
) -> str | None:
    """Download a single image from Imgur."""
    try:
        log.info('Downloading image ({}/{}) {}', post_index + 1, total_posts, url)

        response = await http_client.get(url, headers=IMGUR_HEADERS)
        response.raise_for_status()

        # Extract filename from URL using urllib.parse and pathlib
        parsed_url = urlparse(url)
        filename = Path(parsed_url.path).name

        # If no filename or extension, use a default
        if not filename or '.' not in filename:
            filename = 'image.jpg'

        # Save image file
        image_path = imgur_dir / filename
        image_path.write_bytes(response.content)
        return filename
    except httpx.RequestError as e:
        log.error('Failed to download {}: {}', url, e)
        return None


async def process_extract() -> None:
    client = gel.create_async_client()
    try:
        post_list: list[ListPostContainingImgurResult] = await list_post_containing_imgur(client)
    finally:
        await client.aclose()

    # Early return if no results
    if not post_list:
        log.warning('No posts found containing imgur links')
        return

    # Create directories
    base_dir = Path(tempfile.mkdtemp(prefix='post-migration-'))
    imgur_dir = base_dir / 'imgur'
    imgur_dir.mkdir(parents=True, exist_ok=True)

    # Extract imgur URLs from each post
    processed_results: ExtractionResults = []

    for post in post_list:
        body_content = post.body or ''

        # Find all imgur URLs in the body
        imgur_urls: tuple[str, ...] = tuple(IMGUR_PATTERN.findall(body_content))

        # Convert URLs to HTTPS
        imgur_urls = tuple(url.replace('http://', 'https://') for url in imgur_urls)

        # Create a new entry with images table
        report = PostReport(post.id, tuple(ImageEntry(url) for url in imgur_urls))
        processed_results.append(report)

    # Generate YAML from result using msgspec
    yaml_data: bytes = msgspec.yaml.encode(processed_results)

    # Save YAML file BEFORE downloading images
    yaml_path = base_dir / 'data.yaml'
    yaml_path.write_bytes(yaml_data)

    log.info('YAML data file generated: {}', yaml_path)
    log.info('Starting image downloads...')

    # Track failed files
    failed_files = []

    # HTTP client for downloading images with HTTP/2 support
    async with httpx.AsyncClient(headers=IMGUR_HEADERS, http2=True) as http_client:
        for i, (post, report) in enumerate(zip(post_list, processed_results)):
            imgur_urls = tuple(image.imgur for image in report.images)

            # Download each image
            downloaded_images: list[str] = []
            for url in imgur_urls:
                filename = await download_image(http_client, url, imgur_dir, i, len(post_list))
                if filename:
                    downloaded_images.append(filename)
                else:
                    failed_files.append(url)

    log.info("Files generated in '{}'", base_dir)
    log.info('  - YAML data: {}', yaml_path)
    log.info('  - Imgur images: {}', imgur_dir)
    log.info('Processed {} posts with Imgur images', len(processed_results))
    
    # Final report
    log.info('Download summary: {} successful, {} failed', len(processed_results) - len(failed_files), len(failed_files))
    if failed_files:
        log.info('Failed files:')
        for file in failed_files:
            log.info('  - {}', file)


@cli.command()
@click.option(
    '--input-folder',
    '-I',
    required=True,
    type=click.Path(exists=True, file_okay=False),
    help='Input folder containing YAML and images',
)
@click.option('--bunny-key', '-k', required=True, help='Bunny.net API key')
def replace_imgur_bunny(bunny_key: str, input_folder: str) -> None:
    """Replace Imgur images with Bunny.net CDN URLs."""
    asyncio.run(process_replace_imgur_bunny(input_folder, bunny_key))


async def upload_image_to_bunny(
    bunny_client: httpx.AsyncClient, image_path: Path, imgur_url: str
) -> Result[str, FileNotFoundError | httpx.RequestError]:
    """Upload a single image to Bunny.net and return the CDN URL.

    Args:
        bunny_client: The Bunny.net HTTP client
        image_path: Path to the local image file
        imgur_url: The original Imgur URL

    Returns:
        The Bunny.net CDN URL if successful, None otherwise
    """
    try:
        image_data = image_path.read_bytes()
    except FileNotFoundError as e:
        return Err(e)

    # Generate Bunny.net path with year folder (in UTC)
    current_year = datetime.datetime.now(UTC).year
    # Extract filename from Imgur URL
    parsed_url = urlparse(imgur_url)
    filename = Path(parsed_url.path).name
    bunny_path = f'blogs/{current_year}/{filename}'

    # Bunny.net API setup
    base_url = f'https://{BUNNY_STORAGE_DOMAIN}/{BUNNY_STORAGE_ZONE}'
    upload_url = f'{base_url}/{bunny_path}'

    try:
        response = await bunny_client.put(upload_url, content=image_data)
        response.raise_for_status()

        # Return Bunny.net URL
        bunny_url = f'https://{BUNNY_HOST}/{bunny_path}'
        log.info('Uploaded {} to {}', filename, bunny_url)
        return Ok(bunny_url)
    except httpx.RequestError as e:
        log.error('Failed to upload {}: {}', filename, e)
        return Err(e)


async def update_single_post(gel_client: gel.AsyncIOClient, post_id: UUID, imgur_to_bunny_map: dict[str, str]) -> bool:
    """Update a single post by replacing Imgur URLs with Bunny.net URLs.

    Args:
        gel_client: The Gel database client
        post_id: The UUID of the post to update
        imgur_to_bunny_map: A dictionary mapping Imgur URLs to Bunny.net URLs

    Returns:
        True if the post was successfully updated, False otherwise
    """
    # Get the current post body
    query = """SELECT BlogPost { id, body } FILTER .id = <uuid>$post_id"""
    try:
        result = cast(MiniBlogPost | None, await gel_client.query_single(query, post_id=post_id))
    except gel.errors.EdgeDBError as e:
        log.error('Database error querying post {}: {}', post_id, e)
        return False

    if not result or not result.body:
        log.warning('No body found for post {}', post_id)
        return False

    # Replace Imgur URLs with Bunny URLs in the body
    updated_body = result.body
    replacements_made = 0

    for imgur_url, bunny_url in imgur_to_bunny_map.items():
        if imgur_url and bunny_url:
            updated_body = updated_body.replace(imgur_url, bunny_url)
            replacements_made += 1

    if replacements_made > 0:
        # Update the post in the database
        update_query = """
            UPDATE BlogPost 
            FILTER .id = <uuid>$post_id 
            SET { 
                body := <str>$body,
                html := <str>$html
            }
        """

        # Regenerate HTML from updated body (simplified - in practice you'd use your markdown processor)
        updated_html = updated_body  # Placeholder - replace with actual HTML generation

        try:
            await gel_client.execute(update_query, post_id=post_id, body=updated_body, html=updated_html)
            log.info('Updated post {} with {} replacements', post_id, replacements_made)
            return True
        except gel.errors.EdgeDBError as e:
            log.error('Failed to update post {}: {}', post_id, e)
            return False
    else:
        log.warning('No replacements made for post {}', post_id)
        return False


async def upload_all_images(bunny_client: httpx.AsyncClient, input_path: Path, extractions: Iterable[PostReport]) -> tuple[tuple[PostReport, ...], list[str]]:
    """Upload all images to Bunny.net.

    Returns:
        List of failed upload files
    """
    failed_upload_files = []
    post_complements = []

    # Upload images and update YAML
    for post_info in extractions:
        post_id = post_info.id
        images = post_info.images

        fillup: list[ImageEntry] = []
        for image_entry in images:
            imgur_url = image_entry.imgur
            if not imgur_url:
                continue

            # Extract filename from Imgur URL
            parsed_url = urlparse(imgur_url)
            filename = Path(parsed_url.path).name

            # Check if local image file exists
            image_path = input_path / 'imgur' / filename
            bunny_url = await upload_image_to_bunny(bunny_client, image_path, imgur_url)
            match bunny_url:
                case Ok(u):
                    fillup.append(ImageEntry(imgur_url, u))
                case _:
                    fillup.append(ImageEntry(imgur_url))
                    failed_upload_files.append(imgur_url)
        complement = PostReport(post_id, tuple(fillup))
        post_complements.append(complement)
    return tuple(post_complements), failed_upload_files


async def update_all_posts(gel_client: gel.AsyncIOClient, extractions: Iterable[PostReport]) -> list[str]:
    """Update all posts in the database.

    Returns:
        List of failed update posts
    """
    failed_update_posts = []

    # Update posts in Gel database
    for post_info in extractions:
        post_id = post_info.id
        images = post_info.images

        # Skip if no Bunny URLs were generated
        if not any(image.bunny for image in images):
            continue

        # Create mapping of Imgur to Bunny URLs for this post
        imgur_to_bunny_map = {
            image.imgur: image.bunny for image in images if image.imgur and image.bunny
        }

        log.info('To update post {}', post_id)
        success = await update_single_post(gel_client, post_id, imgur_to_bunny_map)
        if not success:
            failed_update_posts.append(str(post_id))
    
    return failed_update_posts


async def process_replace_imgur_bunny(input_folder: str, bunny_key: str) -> None:
    """Process the replacement of Imgur images with Bunny.net CDN URLs."""
    input_path = Path(input_folder)
    yaml_path = input_path / 'data.yaml'

    if not yaml_path.exists():
        log.error('YAML file not found: {}', yaml_path)
        return

    # Read the YAML data
    yaml_data = yaml_path.read_bytes()
    extractions = msgspec.yaml.decode(yaml_data, type=tuple[PostReport, ...])

    # Bunny.net API setup
    bunny_headers = {'AccessKey': bunny_key, 'Content-Type': 'application/octet-stream'}

    # HTTP client for Bunny.net API
    async with httpx.AsyncClient(headers=bunny_headers, timeout=30.0) as bunny_client:
        # Upload all images
        post_complements, failed_upload_files = await upload_all_images(
            bunny_client, input_path, extractions
        )

    # Save updated YAML
    updated_yaml_data: bytes = msgspec.yaml.encode(post_complements)
    yaml_path.write_bytes(updated_yaml_data)
    log.info('Updated YAML file: {}', yaml_path)

    # Update posts in Gel database
    gel_client = gel.create_async_client()
    try:
        failed_update_posts = await update_all_posts(gel_client, post_complements)
    finally:
        await gel_client.aclose()

    # Calculate successful counts
    total_images = sum(len(post_info.images) for post_info in post_complements)
    successful_uploads = total_images - len(failed_upload_files)
    
    # Count posts that had images to update
    posts_with_images = [post_info for post_info in post_complements if any(image.bunny for image in post_info.images)]
    successful_updates = len(posts_with_images) - len(failed_update_posts)

    # Final report using click.secho
    click.secho('Finished processing Imgur to Bunny.net replacement', fg='green')
    click.secho(f'Upload summary: {successful_uploads} successful, {len(failed_upload_files)} failed', fg='blue')
    if failed_upload_files:
        click.secho('Failed upload files:', fg='red')
        for file in failed_upload_files:
            click.secho(f'  - {file}', fg='red')
    
    click.secho(f'Update summary: {successful_updates} successful, {len(failed_update_posts)} failed', fg='blue')
    if failed_update_posts:
        click.secho('Failed update posts:', fg='red')
        for post_id in failed_update_posts:
            click.secho(f'  - {post_id}', fg='red')


if __name__ == '__main__':
    # Configure colorized logging
    handler = ColorizedStderrHandler(format_string='{record.time:%Y-%m-%d %H:%M:%S} [{record.level_name}] {record.message}')
    with handler:
        cli()
