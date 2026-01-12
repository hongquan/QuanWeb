#!/usr/bin/env python

import asyncio
import re
import tempfile

import click
import gel
import msgspec

from queries.list_post_containing_imgur_async_edgeql import list_post_containing_imgur


@click.command()
def main():
    asyncio.run(process())


async def process():
    client = gel.create_async_client()
    try:
        result_list = await list_post_containing_imgur(client)
    finally:
        await client.aclose()

    # Early return if no results
    if not result_list:
        click.secho('No posts found containing imgur links', fg='yellow')
        return

    # Extract imgur URLs from each post
    processed_results = []
    imgur_pattern = re.compile(r'https?://(?:i\.)?imgur\.com/\S+?(?=\s|"|\'|\)|\]|\}|>|$)')

    for obj in result_list:
        body_content = obj.body or ''
        
        # Find all imgur URLs in the body
        imgur_urls = imgur_pattern.findall(body_content)
        
        # Create a new entry with images table instead of simple imgur_urls list
        processed_obj = {
            'id': obj.id,  # msgspec can serialize UUID directly
            'images': [
                {
                    'imgur': url,
                    'bunny': None  # Placeholder for future bunny.net URLs
                }
                for url in imgur_urls
            ]
        }
        processed_results.append(processed_obj)

    # Generate YAML from result using msgspec
    yaml_data = msgspec.yaml.encode(processed_results)

    # Create temporary file in /tmp/ folder
    temp_file = tempfile.NamedTemporaryFile(suffix='.yaml', dir='/tmp', delete=False)
    temp_file_path = temp_file.name

    with open(temp_file_path, 'wb') as yamlfile:
        yamlfile.write(yaml_data)

    click.secho(f"YAML file '{temp_file_path}' generated with {len(processed_results)} posts", fg='green')


if __name__ == '__main__':
    main()
